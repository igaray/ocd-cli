pub mod config;
pub mod mrn;
pub mod tss;

use crate::ocd::mrn::MassRenameConfig;
use crate::ocd::tss::TimeStampSortConfig;

#[derive(Clone, Debug)]
pub enum Command {
    MassRename { config: MassRenameConfig },
    TimeStampSort { config: TimeStampSortConfig },
    // FixID3 { config: FixID3Config },
    // ElephantClient{ config: ElephantClientConfig },
    // ElephantServer{ config: ElephantServerConfig },
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

// pub fn move_file_to_directory(
//     from: &Path,
//     dest: &Path,
//     _verbosity: Verbosity,
//     dryrun: bool,
//     undo: bool,
// ) -> io::Result<()> {
//     let mut to = PathBuf::new();
//     to.push(dest);
//     to.push(from.file_name().unwrap());
//     println!("Moving \n    {:?} \nto\n    {:?}", from, to);
//     if dryrun {
//         Ok(())
//     } else {
//         if undo {
//             println!("Saving undo information.");
//             panic!("Undo not implemented yet!");
//         }
//         match fs::rename(from, to) {
//             Ok(_) => Ok(()),
//             Err(reason) => {
//                 println!("Error: file {:?} could not be renamed: {:?}", from, reason);
//                 Err(reason)
//             }
//         }
//     }
// }
