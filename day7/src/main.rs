use std::{
    collections::HashMap,
    io::{BufRead, BufReader},
};

#[derive(Clone, Debug)]
struct File {
    size: u64,
}

#[derive(Clone, Default, Debug)]
struct Directory {
    children: HashMap<String, DirectoryEntry>,
    total_size: u64,
}
impl Directory {
    fn add_entry(&mut self, path_components: &[String], entry: DirectoryEntry) {
        self.total_size += match entry {
            DirectoryEntry::File(ref file) => file.size,
            DirectoryEntry::Directory(ref directory) => directory.total_size,
        };

        if path_components.len() == 1 {
            self.children.insert(path_components[0].to_string(), entry);
        } else {
            let (first, remainder) = path_components.split_first().unwrap();
            let child = self.children.get_mut(first).unwrap();
            let DirectoryEntry::Directory(ref mut child) = child else {
                panic!("directory {} not found", first);
            };
            child.add_entry(remainder, entry);
        }
    }

    fn each_dir(&self, predicate: &mut dyn FnMut(&Directory)) {
        predicate(self);

        for (_, child) in &self.children {
            if let DirectoryEntry::Directory(ref dir) = child {
                dir.each_dir(predicate);
            }
        }
    }
}
#[derive(Clone, Debug)]
enum DirectoryEntry {
    File(File),
    Directory(Directory),
}

#[derive(Clone, Debug)]
enum Command {
    CdUp,
    CdDown {
        destination: String,
    },
    Ls {
        results: HashMap<String, DirectoryEntry>,
    },
}
impl Command {
    fn parse(lines: &[String]) -> Command {
        let (input, output) = lines.split_first().unwrap();
        let input = input.strip_prefix("$ ").unwrap();
        match input {
            "ls" => Command::Ls {
                results: output
                    .iter()
                    .map(|output_line| {
                        let mut words = output_line.split_ascii_whitespace();
                        let first = words.next().unwrap();
                        let name = words.next().unwrap();

                        (
                            name.to_string(),
                            if first == "dir" {
                                DirectoryEntry::Directory(Directory::default())
                            } else {
                                DirectoryEntry::File(File {
                                    size: first.parse().unwrap(),
                                })
                            },
                        )
                    })
                    .collect(),
            },
            "cd .." => Command::CdUp,
            command if command.starts_with("cd ") => {
                let destination = command.strip_prefix("cd ").unwrap().to_string();
                Command::CdDown { destination }
            }
            _ => panic!("unknown command"),
        }
    }
}

fn main() {
    let lines: Result<Vec<_>, _> = BufReader::new(std::fs::File::open("input.txt").unwrap())
        .lines()
        .collect();
    let lines = lines.unwrap();

    let mut cur_command = Vec::new();
    let mut commands = Vec::new();
    for line in lines.iter().skip(1) {
        if line.starts_with("$") {
            if !cur_command.is_empty() {
                commands.push(Command::parse(&cur_command[..]));
            }
            cur_command.clear();
        }

        cur_command.push(line.clone());
    }
    if !cur_command.is_empty() {
        commands.push(Command::parse(&cur_command[..]));
    }

    let mut cwd = Vec::new();
    let mut root_dir = Directory::default();

    for command in commands {
        match command {
            Command::CdUp => {
                cwd.pop().unwrap();
            }
            Command::CdDown { destination } => {
                cwd.push(destination);
            }
            Command::Ls { results } => {
                for (name, entry) in results {
                    cwd.push(name);
                    root_dir.add_entry(cwd.as_slice(), entry);
                    cwd.pop();
                }
            }
        }
    }

    let total_disk_size = 70_000_000;
    let req_disk_space = 30_000_000;
    let free_disk_space = total_disk_size - root_dir.total_size;
    let delete_threshold = dbg!(req_disk_space - free_disk_space);

    // dbg!(&root_dir);

    // let mut total_size_under_100k = 0;
    // root_dir.each_dir(&mut |dir| if dir.total_size <= 100_000 {
    //     total_size_under_100k += dir.total_size;
    // });
    // println!("{}", total_size_under_100k);

    let mut smallest_dir_larger_than_threshold = root_dir.total_size;
    root_dir.each_dir(&mut |dir| {
        if dir.total_size >= delete_threshold && dir.total_size < smallest_dir_larger_than_threshold {
            smallest_dir_larger_than_threshold = dir.total_size;
        }
    });
    println!("{}", smallest_dir_larger_than_threshold);
}
