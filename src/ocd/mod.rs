pub mod config;
pub mod mrn;
pub mod tss;

use crate::ocd::mrn::MassRenameConfig;
use crate::ocd::tss::TimeStampSortConfig;

/// The Command enum represents which subcommand ocd will run and carries its
/// configuration with it.
#[remain::sorted]
#[derive(Clone, Debug)]
pub enum Command {
    MassRename { config: MassRenameConfig },
    TimeStampSort { config: TimeStampSortConfig },
    // FixID3 { config: FixID3Config },
    // ElephantClient{ config: ElephantClientConfig },
    // ElephantServer{ config: ElephantServerConfig },
}
