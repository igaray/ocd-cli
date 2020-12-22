pub mod config;
pub mod id3;
pub mod input;
pub mod mrn;
pub mod output;
pub mod tss;

use crate::ocd::id3::FixId3Config;
use crate::ocd::mrn::MassRenameConfig;
use crate::ocd::tss::TimeStampSortConfig;

/// The Command enum represents which subcommand ocd will run and carries its
/// configuration with it.
#[remain::sorted]
#[derive(Clone, Debug)]
pub enum Command {
    FixId3 { config: FixId3Config },
    MassRename { config: MassRenameConfig },
    TimeStampSort { config: TimeStampSortConfig },
    // ElephantClient{ config: ElephantClientConfig },
    // ElephantServer{ config: ElephantServerConfig },
}
