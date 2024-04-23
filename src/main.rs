use std::{env, process, fs};
use clap::{Args, Parser, Subcommand};
use toml::Table;
use anyhow::{Result, anyhow};

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
    /// Build kernel
    Build,
    /// Run kernel
    Run,
    /// Get module from repo and modify it locally
    Get(ModArgs),
    /// Put module back to repo
    Put(ModArgs),
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
    /// Name of this project
    name: String,
}

fn main() {
    let cli = Cli::parse();

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Commands::New(args) => {
            create_project(args)
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
    }.unwrap_or_else(|e| {
        println!("fatal error: {:?}", e);
    });
}

fn build() -> Result<()> {
    let mut child = process::Command::new("make").spawn()?;
    child.wait()?;
    Ok(())
}

fn run() -> Result<()> {
    let mut child = process::Command::new("make").arg("run").spawn()?;
    child.wait()?;
    Ok(())
}

fn create_project(args: &NewArgs) -> Result<()> {
    println!("new {} --root {}", args.name, args.root);
    let tool_path = get_tool_path().unwrap();
    let tpl_files = tool_path + "/tpl_files/*";
    println!("Path of this executable is: {}", tpl_files);
    fs::create_dir(&args.name)?;
    let cp_cmd = format!("cp -r {} ./{}/", tpl_files, &args.name);
    let _output = process::Command::new("sh").arg("-c").arg(cp_cmd).output()?;
    println!("Create proj ok!");
    Ok(())
}

fn get(args: &ModArgs) -> Result<()> {
    let name = &args.name;
    let url = get_mod_url(name)?;

    let mut child = process::Command::new("git").arg("clone").arg(&url).spawn()?;
    child.wait()?;

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
        toml::Value::String(format!("./{}", name)),
    );
    fs::write("Cargo.toml", toml::to_string(&cargo_toml)?)?;
    Ok(())
}

fn put(args: &ModArgs) -> Result<()> {
    let name = &args.name;
    let url = get_mod_url(name)?;

    let child = process::Command::new("git")
                    .arg("status")
                    .arg("-s")
                    .current_dir(format!("./{}", name))
                    .stdout(process::Stdio::piped())
                    .spawn()?;
    let output = child.wait_with_output()?;
    if output.stdout.len() != 0 {
        println!("{}", String::from_utf8(output.stdout.clone())?);
        return Err(anyhow!("There're some files modified, please handle them first."));
    }

    let mut cargo_toml: Table = toml::from_str(&fs::read_to_string("Cargo.toml")?)?;
    let patch_table = cargo_toml.get_mut("patch").unwrap().as_table_mut().unwrap();
    patch_table.remove(&url);
    fs::write("Cargo.toml", toml::to_string(&cargo_toml)?)?;

    fs::remove_dir_all(format!("./{}", name))?;
    Ok(())
}

fn get_mod_url(name: &str) -> Result<String> {
    let repo_toml: Table = toml::from_str(&fs::read_to_string("Repo.toml")?)?;
    let mod_list = repo_toml.get("mod_list").unwrap();
    let url = mod_list.get(name).unwrap().as_str().unwrap();
    Ok(remove_quotes(url))
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

/*
use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    name: String,

    /// Number of times to greet
    #[arg(short, long, default_value_t = 1)]
    count: u8,
}

fn main() {
    let args = Args::parse();

    for _ in 0..args.count {
        println!("Hello {}!", args.name)
    }
}
*/
