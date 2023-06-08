pub mod tree_generator {
    use regex::Regex;
    use std::fs::{self, Metadata};
    use std::path::PathBuf;

    pub struct TreeGeneratorConfig {
        pub root_files: Vec<String>,
        /// Include file metadata, makes process slow
        pub include_metadata: bool,
        /// Only include directories
        pub directories_only: bool,
        /// List all files, also hidden files starting with a "."
        pub all_files: bool,
        /// Regex to exclude files
        pub regex_exclude_files: Regex,
        /// Apply regex?
        pub use_regex: bool,
    }

    impl TreeGeneratorConfig {
        pub fn default() -> TreeGeneratorConfig {
            TreeGeneratorConfig {
                root_files: vec!["./".to_string()],
                include_metadata: false,
                directories_only: false,
                all_files: false,
                regex_exclude_files: Regex::new(r"").unwrap(),
                use_regex: false,
            }
        }
    }

    pub struct TreeGeneratorStats {
        pub files: u16,
        pub directories: u16,
        pub ignored: u16,
    }

    impl TreeGeneratorStats {
        pub fn new() -> TreeGeneratorStats {
            return TreeGeneratorStats {
                files: 0,
                directories: 0,
                ignored: 0,
            };
        }
    }

    pub struct TreeGenerator {
        config: TreeGeneratorConfig,
        pub stats: TreeGeneratorStats,
    }

    impl TreeGenerator {
        pub fn generate(&mut self, config: TreeGeneratorConfig) -> Tree {
            self.config = config;
            self.stats = TreeGeneratorStats::new();
            self.create_tree_for_files()
        }

        fn create_tree_for_files(&mut self) -> Tree {
            let mut tree = Tree { files: Vec::new() };

            for file in self.config.root_files.clone().into_iter() {
                let tree_item = self.list_files_recursive(&file, &"".to_string(), true);
                if tree_item.is_some() {
                    tree.files.push(tree_item.unwrap());
                }
            }

            tree
        }

        fn list_files_recursive(
            &mut self,
            path: &String,
            file: &String,
            is_root_iteration: bool,
        ) -> Option<TreeItem> {
            let file_path = path.to_owned() + "/" + &file;

            // hidden files only with all option
            if file.starts_with('.') && !self.config.all_files && !is_root_iteration {
                self.stats.ignored += 1;
                return None;
            }

            // if regex is defined filter by regex
            if self.config.use_regex {
                if self.config.regex_exclude_files.is_match(file) {
                    self.stats.ignored += 1;
                    println!("{}    {}", file, is_root_iteration);

                    return None;
                }
            }

            if self.is_dir(&file_path) {
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
                if self.config.directories_only {
                    self.stats.ignored += 1;
                    return None;
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

        pub fn new() -> TreeGenerator {
            return TreeGenerator {
                config: TreeGeneratorConfig::default(),
                stats: TreeGeneratorStats::new(),
            };
        }

        fn needs_metadata(&self) -> bool {
            self.config.include_metadata
        }

        fn collect_metadata(&self, file_path: &String) -> Option<Metadata> {
            if self.needs_metadata() {
                return fs::metadata(&file_path).ok();
            }
            None
        }
    }

    pub struct TreeItem {
        pub file: String,
        pub metadata: Option<Metadata>,
        pub files: Vec<TreeItem>,
        pub is_dir: bool,
    }

    pub struct Tree {
        pub files: Vec<TreeItem>,
    }
}
