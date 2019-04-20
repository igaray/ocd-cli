pub mod config;
pub mod mrn;
pub mod tss;

use crate::ocd::mrn::MassRenameConfig;
use crate::ocd::tss::TimeStampSortConfig;

#[derive(Clone, Debug)]
pub enum Command {
    MassRename{config: MassRenameConfig},
    TimeStampSort{config: TimeStampSortConfig},
}

// pub fn create_directory(args: &clap::ArgMatches, directory: &Path) -> io::Result<()> {
//     if !args.is_present("dry-run") {
//         let mut full_path = PathBuf::new();
//         full_path.push(directory);
//         match fs::create_dir(full_path) {
//             Ok(_) => {
//                 Ok(())
//             },
//             Err(reason) => {
//                 match reason.kind() {
//                     io::ErrorKind::AlreadyExists => {
//                         Ok(())
//                     },
//                     _ => {
//                         if !args.is_present("silent") {
//                             println!("Error: directory could not be created: {:?}", reason.kind());
//                         }
//                         Err(reason)
//                     }
//                 }
//             },
//         }
//     }
//     else {
//         Ok(())
//     }
// }

// pub fn move_file(config: &Config, from: &Path, dest: &Path) -> io::Result<()> {
//     //     if verbose print before and after
//     //     if undo and successfull add to undo file
//     let mut to = PathBuf::new();
//     to.push(dest);
//     to.push(from.file_name().unwrap());
//     if let Verbosity::Debug = config.verbosity {
//         println!("Moving '{:?}' to '{:?}'", from, to)
//     }
//     // if !args.is_present("dry-run") {
//         match fs::rename(from, to) {
//             Ok(_) => {
//                 // if config.undo {
//                 //     if !args.is_present("silent") {
//                 //         println!("Saving undo information.");
//                 //     }
//                 // }
//                 Ok(())
//             },
//             Err(reason) => {
//                 // if !args.is_present("silent") {
//                 //     println!("Error: file {:?} could not be renamed: {:?}", from, reason);
//                 // }
//                 Err(reason)
//             },
//         }
//     // }
//     // else {
//     //     Ok(())
//     // }
// }
