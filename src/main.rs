use clap::Parser;
use colored::*;
use std::fs::{self, Metadata};
use std::path::PathBuf;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::fs::FileTypeExt;
use std::str::FromStr;
use chrono::prelude::*;
use regex::Regex;

const TREE_DATA_SPACE: &str = "  ";

const COLOR_GRAY: Color = Color::TrueColor { r: 69, g: 69, b: 69 };
const COLOR_DIR: Color = Color::Blue;
const COLOR_FILE: Color = Color::White;
const COLOR_PERMISSIONS: Color = COLOR_GRAY;
const COLOR_SIZE: Color = COLOR_GRAY;
const COLOR_IGNORED: Color = Color::Red;
const COLOR_LAST_MODIFIED: Color = COLOR_GRAY;

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

    let config = TreeGeneratorConfig::from_args(&args);
    let mut tree_generator = TreeGenerator::new(config);
    let tree = tree_generator.generate();

    let config_renderer = TreeGeneratorConfig::from_args(&args);
    let renderer = TreeRendererCLI::new(tree, config_renderer);

    println!("");
    renderer.render();
    println!("");

    println!("directories: {}  files: {}  ignored: {}", 
        tree_generator.stats.directories.to_string().color(COLOR_DIR),
        tree_generator.stats.files.to_string().color(COLOR_FILE), 
        tree_generator.stats.ignored.to_string().color(COLOR_IGNORED)
    )
}

fn parse_regex(regex_string: &String) -> Option<Regex> {
    if regex_string == "" {
        return None
    }

    let regex = Regex::from_str(regex_string);
    if regex.is_ok() {
        Some(regex.unwrap())
    } else {
        None
    }
}

struct TreeGeneratorConfig {
    root: Vec<String>,
    block_size: String,
    long: bool,
    dir_only: bool,
    all: bool,
    regex: Regex,
    use_regex: bool,
    reverse: bool,
    tree_spacer: String,
}

impl TreeGeneratorConfig {
    fn from_args(args: &Args) -> TreeGeneratorConfig {
        let mut use_regex = true;
        let regex = parse_regex(&args.exclude).unwrap_or_else(|| {
            use_regex = false;
            Regex::new(r"").unwrap()
        });
        return TreeGeneratorConfig {
            root: args.file.clone(),
            block_size: args.block_size.clone(),
            long: args.long.clone(),
            dir_only: args.dir_only.clone(),
            all: args.all.clone(),
            regex,
            use_regex,
            reverse: args.reverse.clone(),
            tree_spacer: args.tree_spacer.clone(),
        };
    }
}

struct TreeGeneratorStats {
    files: u16,
    directories: u16,
    ignored: u16,
}

impl TreeGeneratorStats {
    fn new() -> TreeGeneratorStats {
        return TreeGeneratorStats {
            files: 0,
            directories: 0,
            ignored: 0
        }
    }
}

struct TreeGenerator {
    config: TreeGeneratorConfig,
    stats: TreeGeneratorStats,
}

impl TreeGenerator {

    fn generate(&mut self) -> Tree {
        self.stats = TreeGeneratorStats::new();
        self.create_tree_for_files()
    }

    fn create_tree_for_files(&mut self) -> Tree {
        let mut tree = Tree { files: Vec::new() };

        for file in self.config.root.clone().into_iter() {
            let tree_item = self.list_files_recursive(&file, &"".to_string(), true);
            if tree_item.is_some() {
                tree.files.push(tree_item.unwrap());
            }
        }

        tree
    }

    fn list_files_recursive(&mut self, path: &String, file: &String, is_root_iteration: bool) -> Option<TreeItem> {
        let file_path = path.to_owned() + "/" + &file;

        // hidden files only with all option
        if file.starts_with('.') && !self.config.all && !is_root_iteration {
            self.stats.ignored += 1;
            return None
        }

        // if regex is defined filter by regex
        if self.config.use_regex {
            if self.config.regex.is_match(file) {
                self.stats.ignored += 1;
                println!("{}    {}", file, is_root_iteration);

                return None
            }
        }

        if self.is_dir(&file_path)  {
            self.stats.directories += 1;
            let mut tree_item = TreeItem {
                file: file.to_owned(),
                files: Vec::new(),
                metadata: self.collect_metadata(&file_path),
                is_dir: true,
            };

            let files = fs::read_dir(&file_path).unwrap();

            for sub_file in files {
                let f = sub_file.unwrap().file_name().into_string().unwrap();
                let sub_tree_item = self.list_files_recursive(&file_path, &f, false);
                
                if sub_tree_item.is_some() {
                    tree_item.files.push(sub_tree_item.unwrap())
                }
            }
            Some(tree_item)
            
        } else {
            // dont return files if mode is dir only
            if self.config.dir_only {
                self.stats.ignored += 1;
                return None
            }

            self.stats.files += 1;
            Some(TreeItem {
                file: file.to_owned(),
                files: Vec::new(),
                metadata: self.collect_metadata(&file_path),
                is_dir: false,
            })
        }
    }

    fn is_dir(&self, file: &String) -> bool {
        PathBuf::from(file).is_dir()
    }

    fn new(config: TreeGeneratorConfig) -> TreeGenerator {
        return TreeGenerator { 
            config,
            stats: TreeGeneratorStats::new()
         };
    }

    fn needs_metadata(&self) -> bool {
        self.config.long
    }

    fn collect_metadata(&self, file_path: &String) -> Option<Metadata> {
        if self.needs_metadata() {
            return fs::metadata(&file_path).ok();
        }
        None
    }
}

struct TreeItem {
    file: String,
    metadata: Option<Metadata>,
    files: Vec<TreeItem>,
    is_dir: bool,
}

struct Tree {
    files: Vec<TreeItem>,
}

struct TreeRendererCLI {
    config: TreeGeneratorConfig,
    tree: Tree,
}

impl TreeRendererCLI {
    fn new(tree: Tree, config: TreeGeneratorConfig) -> TreeRendererCLI {
        TreeRendererCLI { config, tree }
    }

    fn render(&self) {
        for tree_item in self.tree.files.iter() {
            self.render_recursive(tree_item, 0)
        }
    }

    fn render_recursive(&self, tree_item: &TreeItem, depth: usize) {
        for sub_tree_item in tree_item.files.iter() {
            if self.config.reverse {
                self.render_recursive(&sub_tree_item, depth + 1);
                self.render_tree_item(&sub_tree_item, &depth);
            } else {
                self.render_tree_item(&sub_tree_item, &depth);
                self.render_recursive(&sub_tree_item, depth + 1);
            }
            
        }
    }

    fn render_tree_item(&self, sub_tree_item: &TreeItem, depth: &usize) {
        print!("{}", self.config.tree_spacer.repeat(*depth));

        if sub_tree_item.is_dir {
            print!("{}", sub_tree_item.file.color(COLOR_DIR));
        } else {
            print!("{}", sub_tree_item.file.color(COLOR_FILE));
        }

        if self.config.long {
            print!(
                "{}{}{}{}{}{}[{}]",
                TREE_DATA_SPACE,
                TREE_DATA_SPACE,
                self.format_file_permissions(sub_tree_item).color(COLOR_PERMISSIONS),
                TREE_DATA_SPACE,
                self.format_file_size(sub_tree_item).color(COLOR_SIZE),
                TREE_DATA_SPACE,
                self.format_last_modified(sub_tree_item).color(COLOR_LAST_MODIFIED)
            )
        }

        print!("\n")
    }

    fn format_file_size(&self, tree_item: &TreeItem) -> String {
        if let Some(metadata) = &tree_item.metadata {
            let size = match self.config.block_size.as_str() {
                "G" | "g" => metadata.len() / 1024 / 1024 / 1024,
                "M" | "m" => metadata.len() / 1024 / 1024,
                "K" | "k" => metadata.len() / 1024,
                _ => metadata.len(),
            };

            let size_unit = match self.config.block_size.as_str() {
                "G" | "g" => "G",
                "M" | "m" => "M",
                "K" | "k" => "K",
                _ => "B",
            };
            size.to_string().to_owned() + size_unit
            // ...
        } else {
            "-".to_string()
        }
    }

    fn format_file_permissions(&self, tree_item: &TreeItem) -> String {
        if let Some(metadata) = &tree_item.metadata {
            let mode = metadata.permissions().mode();
            let file_type = if metadata.is_symlink() {
                "l"
            } else if metadata.is_dir() {
                "d"
            } else if metadata.file_type().is_char_device() {
                "c"
            } else if metadata.file_type().is_block_device() {
                "b"
            } else if metadata.file_type().is_fifo() {
                "p"
            } else if metadata.file_type().is_socket() {
                "s"
            } else {
                "-"
            };

            format!("{}{:o}", file_type, mode)
        } else {
            "-".to_string()
        }
    }

    fn format_last_modified(&self, tree_item: &TreeItem) -> String {
        if let Some(metadata) = &tree_item.metadata {
            let last_modified = metadata.modified();
            if last_modified.is_ok() {
                let dt: DateTime<Utc> = last_modified.unwrap().into();
                dt.format("%Y-%m-%d %H:%M:%S").to_string()
            } else {
                "-".to_string()
            }
        } else {
            "-".to_string()
        }
    }
}
