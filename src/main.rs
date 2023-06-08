use clap::Parser;
use std::str::FromStr;
use regex::Regex;
use colored::*;

// modules
mod tree_generator;
mod tree_renderer;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The target file(s) or directories
    #[clap(value_name = "File", index = 1, default_value = "./", num_args = 1..)]
    file: Vec<String>,

    #[clap(short, long, value_name = "Block size", default_value = "")]
    block_size: String,

    #[clap(short, long, value_name = "Long", default_value = "false")]
    long: bool,

    #[clap(short, long, value_name = "All", default_value = "false")]
    all: bool,

    #[clap(short, long, value_name = "Directory only", default_value = "false")]
    dir_only: bool,

    #[clap(short, long, value_name = "Reverse tree", default_value = "false")]
    reverse: bool,

    #[clap(short, long, value_name = "Ignore files base on regex", default_value = "")]
    exclude: String,

    #[clap(short, long, value_name = "Tree spacer", default_value = "  ")]
    tree_spacer: String,
}

fn main() {
    let args = Args::parse();

    let config = tree_generator::tree_generator::TreeGeneratorConfig{
        root_files: args.file,
        include_metadata: args.long,
        directories_only: args.dir_only,
        all_files: args.all,
        regex_exclude_files: parse_regex(&args.exclude),
        use_regex: args.exclude != ""
    };

    let mut tree_generator = tree_generator::tree_generator::TreeGenerator::new();
    let tree = tree_generator.generate(config);

    let config_renderer = tree_renderer::tree_renderer::TreeRendererConfig {
        tree_spacer: args.tree_spacer,
        display_permissions: args.long,
        display_file_size: args.long,
        display_last_modified: args.long,
        reverse_bottom_top: args.reverse,
        block_size: args.block_size,
        color_directory: colored::Color::Blue,
        color_file: colored::Color::White,
        color_file_size: colored::Color::White,
        color_last_modified: colored::Color::White,
        color_permissions: colored::Color::White,
    };
    let mut renderer = tree_renderer::tree_renderer::TreeRendererCLI::new();

    println!("");
    renderer.render(tree, config_renderer);
    println!("");

    println!("directories: {}  files: {}  ignored: {}", 
        tree_generator.stats.directories.to_string().blue(),
        tree_generator.stats.files.to_string().white(), 
        tree_generator.stats.ignored.to_string().red()
    )
}

fn parse_regex(regex_string: &String) -> Regex {
    let regex = Regex::from_str(regex_string);
    if regex.is_ok() {
        regex.unwrap()
    } else {
        Regex::new(r"").unwrap()
    }
}


