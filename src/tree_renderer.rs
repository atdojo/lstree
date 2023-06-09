pub mod tree_renderer {
    use crate::tree_generator::tree_generator::*;
    use chrono::prelude::*;
    use colored::*;
    use std::os::unix::fs::FileTypeExt;
    use std::os::unix::fs::PermissionsExt;
    use std::vec;

    pub struct TreeRendererConfig {
        pub tree_spacer: String,
        pub display_permissions: bool,
        pub display_file_size: bool,
        pub display_last_modified: bool,
        pub reverse_bottom_top: bool,
        pub print_all_file: bool,
        /// Unit of file size, only matters with include_metadata
        /// B, K, M, G
        pub block_size: String,
        pub color_file: Color,
        pub color_directory: Color,
        pub color_last_modified: Color,
        pub color_permissions: Color,
        pub color_file_size: Color,
    }

    impl TreeRendererConfig {
        pub fn default() -> TreeRendererConfig {
            TreeRendererConfig {
                tree_spacer: "  ".to_string(),
                display_permissions: false,
                display_file_size: false,
                display_last_modified: false,
                reverse_bottom_top: false,
                print_all_file: false,
                block_size: "B".to_string(),
                color_file: Color::White,
                color_directory: Color::Blue,
                color_last_modified: Color::White,
                color_permissions: Color::White,
                color_file_size: Color::White,
            }
        }
    }

    pub struct TreeRendererCLI {
        config: TreeRendererConfig,
        tree: Tree,
    }

    impl TreeRendererCLI {
        pub fn new() -> TreeRendererCLI {
            TreeRendererCLI {
                config: TreeRendererConfig::default(),
                tree: Tree {
                    files: vec![TreeItem {
                        files: vec![],
                        file: "".to_string(),
                        metadata: None,
                        is_dir: false,
                    }],
                },
            }
        }

        pub fn render(&mut self, tree: Tree, config: TreeRendererConfig) {
            self.tree = tree;
            self.config = config;
            for tree_item in self.tree.files.iter() {
                self.render_recursive(tree_item, 0, vec![].as_mut())
            }
        }

        fn render_recursive(&self, tree_item: &TreeItem, depth: usize, files_before: &mut Vec<String>) {
            for sub_tree_item in tree_item.files.iter() {
                if self.config.reverse_bottom_top {
                    let mut files_before_clone: Vec<String> = files_before.clone();
                    if sub_tree_item.is_dir {
                        files_before_clone.push(sub_tree_item.file.clone());
                    }
                    self.render_recursive(&sub_tree_item, depth + 1, &mut files_before_clone);
                    self.render_tree_item(&sub_tree_item, &depth, &mut files_before_clone);
                } else {
                    let mut files_before_clone: Vec<String> = files_before.clone();
                    self.render_tree_item(&sub_tree_item, &depth, &mut files_before_clone);
                    if sub_tree_item.is_dir {
                        files_before_clone.push(sub_tree_item.file.clone());
                    }
                    self.render_recursive(&sub_tree_item, depth + 1, &mut files_before_clone);
                }
            }
        }

        fn render_tree_item(&self, sub_tree_item: &TreeItem, depth: &usize, files_before: &mut Vec<String>) {
            if self.config.print_all_file {
                for file in files_before.into_iter() {
                    print!("{}{}", file.color(self.config.color_directory), self.config.tree_spacer)
                }
            } else {
                print!("{}", self.config.tree_spacer.repeat(*depth));
            }

            if sub_tree_item.is_dir {
                print!("{}", sub_tree_item.file.color(self.config.color_directory));
            } else {
                print!("{}", sub_tree_item.file.color(self.config.color_file));
            }

            if self.config.display_permissions {
                print!(
                    "{}{}",
                    self.config.tree_spacer,
                    self.format_file_permissions(sub_tree_item)
                        .color(self.config.color_permissions),
                )
            }

            if self.config.display_file_size {
                print!(
                    "{}{}",
                    self.config.tree_spacer,
                    self.format_file_size(sub_tree_item)
                        .color(self.config.color_file_size),
                )
            }

            if self.config.display_last_modified {
                print!(
                    "{}{}",
                    self.config.tree_spacer,
                    self.format_last_modified(sub_tree_item)
                        .color(self.config.color_last_modified)
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
}
