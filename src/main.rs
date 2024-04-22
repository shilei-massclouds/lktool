use std::env;
use std::io::Result;
use std::process;
use std::fs;
use clap::{Args, Parser, Subcommand};

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
    /// Adds files to myapp
    Add(AddArgs),
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
struct AddArgs {
    name: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Commands::New(args) => {
            create_project(args).unwrap_or_else(|e| {
                panic!("fatal error: {:?}", e);
            })
        },
        Commands::Build => {
            build().unwrap_or_else(|e| {
                panic!("fatal error: {:?}", e);
            })
        },
        Commands::Run => {
            run().unwrap_or_else(|e| {
                panic!("fatal error: {:?}", e);
            })
        },
        Commands::Add(name) => {
            println!("'myapp add' was used, name is: {:?}", name.name)
        }
    }
}

fn build() -> Result<()> {
    let _output = process::Command::new("make").output()?;
    println!("Build proj ok!");
    Ok(())
}

fn run() -> Result<()> {
    /*
    let output = process::Command::new("make").arg("run").output()?;
    println!("Run proj ok! {:?}", output);
    */
    let mut child = process::Command::new("make").arg("run").spawn()?;
    let _result = child.wait().unwrap();
    println!("Run proj ok!");
    Ok(())
}

fn create_project(args: &NewArgs) -> Result<()> {
    println!("new {} --root {}", args.name, args.root);
    let tool_path = get_tool_path().unwrap();
    let tpl_files = tool_path + "/tpl_files/*";
    println!("Path of this executable is: {}", tpl_files);
    fs::create_dir(&args.name)?;
    //let output = process::Command::new("ls").arg(tpl_files).output()?;
    let cp_cmd = format!("cp -r {} ./{}/", tpl_files, &args.name);
    let _output = process::Command::new("sh").arg("-c").arg(cp_cmd).output()?;
    println!("Create proj ok!");
    Ok(())
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