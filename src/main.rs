use std::{env, process, fs, io::Write};
use std::path::Path;
use std::collections::BTreeMap;
use clap::{Args, Parser, Subcommand};
use toml::{Table, Value, map};
use anyhow::{Result, anyhow};

const DEFAULT_ARCH_FILE: &str = ".default_arch";
const DEFAULT_ARCH: &str = "riscv64";
const ROOT_FILE: &str = ".root";
const BTP_URL: &str = "git@github.com:shilei-massclouds/btp.git";
const LOCAL_MODE: &str = ".local_mode";

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new kernel project
    New(NewArgs),
    /// Prepare for this project
    Prepare,
    /// List available common modules and root modules
    List(ListArgs),
    /// Config kernel
    Config(ConfigArgs),
    /// Build kernel
    Build,
    /// Run kernel
    Run,
    /// Get module from repo and modify it locally
    Get(ModArgs),
    /// Put module back to repo
    Put(ModArgs),
    /// Make dependency graph
    DepGraph,
    /// Check patched modules status
    Status,
    /// Change root of the project
    Chroot(RootArgs),
}

#[derive(Args)]
struct NewArgs {
    /// Name of this project
    name: String,
    /// Root component of this project
    #[arg(long)]
    root: String,
}

#[derive(Args)]
struct ModArgs {
    /// Name of target component
    name: String,
}

#[derive(Args)]
struct RootArgs {
    /// Root component of project
    root: Option<String>,
}

#[derive(Args)]
struct ConfigArgs {
    /// Arch: one of ["x86_64", "aarch64", "riscv64", "loongarch64", "um"]
    arch: String,
    /// Config: one of ["blk"]
    conf: Option<String>,
}

#[derive(Args)]
struct ListArgs {
    /// Class of modules (e.g. root, ..)
    #[arg(short)]
    class: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::New(args) => {
            create_project(args)
        },
        Commands::Prepare => {
            prepare()
        },
        Commands::List(args) => {
            list(args)
        },
        Commands::Config(args) => {
            config(args)
        },
        Commands::Build => {
            build()
        },
        Commands::Run => {
            run()
        },
        Commands::Get(args) => {
            get(args)
        },
        Commands::Put(args) => {
            put(args)
        },
        Commands::DepGraph => {
            depgraph()
        },
        Commands::Status => {
            status()
        },
        Commands::Chroot(args) => {
            chroot(args)
        },
    }.unwrap_or_else(|e| {
        println!("{:?}", e);
    });
}

fn list(args: &ListArgs) -> Result<()> {
    let tool_path = get_tool_path().unwrap();
    let repo_path = format!("{}/tpl_files/Repo.toml", tool_path);
    let repo_toml: Table = toml::from_str(&fs::read_to_string(repo_path)?)?;
    let list_name = if let Some(ref class) = args.class {
        assert!(class == "root", "Now only support 'root'");
        format!("{}_list", class)
    } else {
        "mod_list".to_string()
    };
    let list = repo_toml.get(&list_name).unwrap();
    for name in list.as_table().unwrap().keys() {
        println!("{}", name);
    }
    Ok(())
}

fn config(args: &ConfigArgs) -> Result<()> {
    assert!(matches!(args.arch.as_str(),
        "x86_64" | "aarch64" | "riscv64" | "loongarch64" | "um"
    ));
    fs::write(DEFAULT_ARCH_FILE, &args.arch)?;
    Ok(())
}

fn status() -> Result<()> {
    let toml: Table = toml::from_str(&fs::read_to_string("Cargo.toml")?)?;
    let table = toml.get("patch").and_then(|p| p.as_table())
        .ok_or(anyhow!("No patched modules."))?;
    for url in table.keys() {
        let (_, repo) = url.rsplit_once('/').unwrap();
        check_uncommitted_mods(repo)?;
    }
    Ok(())
}

fn build() -> Result<()> {
    let arch = default_arch();
    let conf = parse_conf()?;
    let has_blk = blk_config(&conf);
    let global_cfg = _global_cfg(&conf);
    let mut child = process::Command::new("make")
        .arg(format!("ARCH={}", arch))
        .arg(format!("BLK={}", has_blk))
        .arg(format!("GLOBAL_CFG={}", global_cfg))
        .spawn()?;
    child.wait()?;
    Ok(())
}

fn run() -> Result<()> {
    let arch = default_arch();
    let conf = parse_conf()?;
    let has_blk = blk_config(&conf);
    let global_cfg = _global_cfg(&conf);
    let mut child = process::Command::new("make")
        .arg(format!("ARCH={}", arch))
        .arg(format!("BLK={}", has_blk))
        .arg(format!("GLOBAL_CFG={}", global_cfg))
        .arg("run")
        .spawn()?;
    child.wait()?;
    Ok(())
}

fn _global_cfg(conf: &BTreeMap<String, String>) -> String {
    let mut items : Vec<String> = vec![];
    for (k, v) in conf {
        if v == "n" {
            continue;
        }

        if v == "y" {
            items.push(format!("--cfg={}", k));
        } else {
            items.push(format!("--cfg={}={}", k, v));
        }
    }

    println!("{:?}", items);
    items.join(" ")
}

fn default_arch() -> String {
    if let Ok(arch) = fs::read_to_string(DEFAULT_ARCH_FILE) {
        arch.trim().to_owned()
    } else {
        DEFAULT_ARCH.to_string()
    }
}

fn default_root() -> Option<String> {
    fs::read_to_string(ROOT_FILE).map(|root| {
        root.trim().to_owned()
    }).ok()
}

fn blk_config(conf: &BTreeMap<String, String>) -> String {
    if let Some(v) = conf.get("blk") {
        if v == "y" {
            return v.clone();
        }
    }
    "n".to_owned()
}

fn create_project(args: &NewArgs) -> Result<()> {
    if local_mode() {
        println!("Disable this subcommand in local mode.");
        return Ok(());
    }

    println!("new {} --root {}", args.name, args.root);
    let tool_path = get_tool_path().unwrap();
    let tpl_files = tool_path + "/tpl_files/*";
    fs::create_dir(&args.name)?;
    let cp_cmd = format!("cp -r {} ./{}/", tpl_files, &args.name);
    let _output = process::Command::new("sh").arg("-c").arg(cp_cmd).output()?;

    let url = get_root_url(&args.root, &args.name)?;
    println!("root url: {} -> {}", args.root, url);
    setup_root(&args.root, &url, &args.name)?;

    // Change current directory
    let root = Path::new(&args.name);
    assert!(env::set_current_dir(&root).is_ok());

    // Clone root_component
    _get(&args.root)?;

    // Record location of root mod
    let (_, repo) = url.rsplit_once('/').unwrap();
    fs::write(ROOT_FILE, format!("{}/{}", repo, &args.root))?;

    println!("Create proj ok!");
    Ok(())
}

fn chroot(args: &RootArgs) -> Result<()> {
    let old_root = default_root().unwrap_or("/[unset]".to_owned());
    let (_, old) = old_root.split_once('/').unwrap();

    let new_root = if let Some(ref root) = args.root {
        root
    } else {
        println!("{}", old);
        return Ok(());
    };

    // Todo: check that target mod actually exists in root_list.
    let new = new_root.trim_end_matches('/');
    assert!(new.starts_with("rt_"));

    if !local_mode() {
        _put(old)?;
    }

    let url = get_root_url(&new, ".")?;
    setup_root(&new, &url, ".")?;

    // Clone root_component
    if !local_mode() {
        _get(&new)?;
    }

    // Record location of root mod
    let (_, repo) = url.rsplit_once('/').unwrap();
    fs::write(ROOT_FILE, format!("{}/{}", repo, &new))?;

    println!("chroot: {} -> {}", old, new);
    Ok(())
}

fn prepare() -> Result<()> {
    let conf = parse_conf()?;
    if let Some(v) = conf.get("blk") {
        assert_eq!(v, "y");
        let _ = fs::remove_dir_all("./btp");
        let mut child = process::Command::new("git")
            .arg("clone").arg(&BTP_URL).spawn()?;
        child.wait()?;

        let arch = default_arch();
        let mut child = process::Command::new("make")
            .arg("-C")
            .arg("./btp")
            .arg(format!("ARCH={}", arch))
            .spawn()?;
        child.wait()?;

        let _ = fs::remove_file("./disk.img");
        let mut child = process::Command::new("make")
            .arg("disk_img")
            .spawn()?;
        child.wait()?;

        let mut child = process::Command::new("make")
            .arg("install_apps")
            .arg(format!("ARCH={}", arch))
            .spawn()?;
        child.wait()?;
    }
    Ok(())
}

fn parse_conf() -> Result<BTreeMap<String, String>> {
    let arch = default_arch();
    let root = default_root().unwrap_or_else(|| {
        panic!("Please set root first: 'lk chroot <root_mod>'.");
    });
    let path = format!("{}/defconfig/{}", root, arch);
    let content = if let Ok(content) = fs::read_to_string(&path) {
        content
    } else {
        return Ok(BTreeMap::new());
    };

    let mut conf: BTreeMap<String, String> = BTreeMap::new();
    for line in content.split('\n') {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let (k, v) = line.split_once('=').unwrap();
        conf.insert(k.to_owned(), v.to_owned());
    }
    Ok(conf)
}

fn setup_root(root: &str, url: &str, path: &str) -> Result<()> {
    println!("root url: {} -> {}", root, url);
    let tpl_cargo_path = format!("{}/proj/tpl_Cargo.toml", path);
    let cargo_path = format!("{}/proj/Cargo.toml", path);
    fs::copy(tpl_cargo_path, &cargo_path)?;

    let mut cargo_toml: Table = toml::from_str(&fs::read_to_string(&cargo_path)?)?;
    let dep_table = cargo_toml.get_mut("dependencies").unwrap().as_table_mut().unwrap();
    dep_table.insert(root.to_string(), toml::Value::Table(Table::new()));
    let detail_table = dep_table.get_mut(root).unwrap().as_table_mut().unwrap();
    detail_table.insert(
        String::from("git"),
        toml::Value::String(format!("{}", url)),
    );
    fs::write(&cargo_path, toml::to_string(&cargo_toml)?)?;

    // Append root declaration
    let tpl_path = format!("{}/proj/src/tpl_main.rs", path);
    let code_path = format!("{}/proj/src/main.rs", path);
    fs::copy(tpl_path, &code_path)?;
    let mut code = fs::OpenOptions::new().append(true).open(code_path)?;
    let decl = format!("use {} as root;", root);
    code.write_all(decl.as_bytes())?;
    Ok(())
}

fn get(args: &ModArgs) -> Result<()> {
    let name = args.name.trim_end_matches('/');
    _get(name)
}

fn _get(name: &str) -> Result<()> {
    if local_mode() {
        println!("Disable this subcommand in local mode.");
        return Ok(());
    }

    let url = get_mod_url(name)?;
    let (_, repo) = url.rsplit_once('/').unwrap();
    if fs::metadata(repo).is_err() {
        let mut child = process::Command::new("git").arg("clone").arg(&url).spawn()?;
        child.wait()?;
    }

    let mut cargo_toml: Table = toml::from_str(&fs::read_to_string("Cargo.toml")?)?;
    if !cargo_toml.contains_key("patch") {
        cargo_toml.insert(String::from("patch"), toml::Value::Table(Table::new()));
    }
    let patch_table = cargo_toml.get_mut("patch").unwrap().as_table_mut().unwrap();
    if !patch_table.contains_key(&url) {
        patch_table.insert(url.clone(), toml::Value::Table(Table::new()));
    }
    let url_table = patch_table.get_mut(&url).unwrap().as_table_mut().unwrap();
    url_table.insert(name.to_string(), toml::Value::Table(Table::new()));

    let detail_table = url_table.get_mut(name).unwrap().as_table_mut().unwrap();
    detail_table.insert(
        String::from("path"),
        toml::Value::String(format!("./{}/{}", repo, name))
    );
    fs::write("Cargo.toml", toml::to_string(&cargo_toml)?)?;
    println!("=====================================");
    println!("'{}' is in repo '{}'.", name, repo);
    println!("=====================================");
    Ok(())
}

fn put(args: &ModArgs) -> Result<()> {
    let name = args.name.trim_end_matches('/');
    _put(name)
}

fn _put(name: &str) -> Result<()> {
    if local_mode() {
        println!("Disable this subcommand in local mode.");
        return Ok(());
    }

    let url = get_mod_url(name)?;
    let (_, repo) = url.rsplit_once('/').unwrap();
    if fs::metadata(repo).is_err() {
        println!("repo '{}' doesn't exists!", repo);
        return Ok(());
    }

    check_uncommitted_mods(repo)?;

    let mut cargo_toml: Table = toml::from_str(&fs::read_to_string("Cargo.toml")?)?;
    let patch_table = cargo_toml.get_mut("patch").unwrap().as_table_mut().unwrap();
    patch_table.remove(&url);
    fs::write("Cargo.toml", toml::to_string(&cargo_toml)?)?;

    fs::remove_dir_all(format!("./{}", repo))?;
    Ok(())
}

fn check_uncommitted_mods(repo: &str) -> Result<()> {
    let child = process::Command::new("git")
                    .arg("status")
                    .arg("-s")
                    .current_dir(format!("./{}", repo))
                    .stdout(process::Stdio::piped())
                    .spawn()?;
    let output = child.wait_with_output()?;
    if output.stdout.len() != 0 {
        println!("{}:\n{}", repo, String::from_utf8(output.stdout.clone())?);
        return Err(anyhow!("Some files modified, please handle them first."));
    }
    let child = process::Command::new("git")
                    .arg("diff")
                    .arg("@{u}")
                    .arg("--stat")
                    .current_dir(format!("./{}", repo))
                    .stdout(process::Stdio::piped())
                    .spawn()?;
    let output = child.wait_with_output()?;
    if output.stdout.len() != 0 {
        println!("{}:\n{}", repo, String::from_utf8(output.stdout.clone())?);
        return Err(anyhow!("Some files unpushed, please handle them first."));
    }
    Ok(())
}

fn get_mod_url(name: &str) -> Result<String> {
    let repo_toml: Table = toml::from_str(&fs::read_to_string("Repo.toml")?)?;
    let mod_list = repo_toml.get("mod_list").unwrap();
    let url = if let Some(url) = mod_list.get(name) {
        url
    } else {
        let root_list = repo_toml.get("root_list").unwrap();
        root_list.get(name).ok_or(anyhow!("no {} in mod_list and root_list", name))?
    };
    let mut url = remove_quotes(url.as_str().unwrap());
    if url.find('/').is_none() {
        let org = get_default_org(&repo_toml)?;
        url = format!("{}/{}", org, url);
    }
    Ok(url)
}

fn get_root_url(name: &str, path: &str) -> Result<String> {
    let repo_path = format!("{}/Repo.toml", path);
    let repo_toml: Table = toml::from_str(&fs::read_to_string(repo_path)?)?;
    let root_list = repo_toml.get("root_list").unwrap();
    let url = root_list.get(name).ok_or(anyhow!("no {} in root_list", name))?;
    let mut url = remove_quotes(url.as_str().unwrap());
    if url.find('/').is_none() {
        let org = get_default_org(&repo_toml)?;
        url = format!("{}/{}", org, url);
    }
    Ok(url)
}

fn get_default_org(toml: &map::Map<String, Value>) -> Result<String> {
    let default = toml.get("default").unwrap();
    let org = default.get("org").unwrap();
    Ok(remove_quotes(org.as_str().unwrap()))
}

fn remove_quotes(s: &str) -> String {
    s.trim_matches(|c| c == '\"' || c == '\'').to_string()
}

fn get_tool_path() -> Option<String> {
    // Note: in dep-mod, lktool is at '[tool_path]/target/debug/'.
    // And template-files are just at '[tool_path]/'.
    // So funny?! Refine this function.
    let path = env::current_exe().ok()?;
    let path = path.parent()?.parent()?.parent()?;
    Some(path.to_str()?.to_owned())
}

fn depgraph() -> Result<()> {
    let cmd = "cargo depgraph --root proj --hide boot | dot -Tpng > depgraph.png";
    let _output = process::Command::new("sh").arg("-c").arg(cmd).output()?;
    Ok(())
}

fn local_mode() -> bool {
    fs::metadata(LOCAL_MODE).is_ok()
}
