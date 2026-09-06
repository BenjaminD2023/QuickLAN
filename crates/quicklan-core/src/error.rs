use serde::Serialize;

/// Messages are constants. Never interpolate invitations, upstream output or OS errors.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, thiserror::Error)]
#[serde(rename_all = "snake_case")]
pub enum Error {
    #[error("The invitation is malformed or too large.")]
    InvalidInvitation,
    #[error("This invitation requires an unsupported version of QuickLAN.")]
    UnsupportedVersion,
    #[error("Use a name between 1 and 64 characters, without control characters.")]
    InvalidLabel,
    #[error("Choose a canonical private IPv4 /24 subnet.")]
    InvalidSubnet,
    #[error("The proposed address range overlaps another network. All members must agree on a replacement range.")]
    RouteConflict,
    #[error("Use a TCP or UDP endpoint with an explicit port, without credentials, paths, queries or fragments.")]
    InvalidEndpoint,
    #[error("Direct-only traffic is unavailable until enforcement and path migration tests pass.")]
    UnsupportedPolicy,
    #[error("This policy requires an explicit, shared node and acceptance of its assistance and relay behavior.")]
    AssistanceConsentRequired,
    #[error("Review and accept the invitation before saving this network.")]
    ConfirmationRequired,
    #[error("This network is already saved. Forgetting it is a separate local action.")]
    AlreadySaved,
    #[error("The network was not found.")]
    NotFound,
    #[error("Disconnect the active network before changing or forgetting it.")]
    Busy,
    #[error("Secure storage could not be accessed. Unlock or allow access to the operating system credential store and try again.")]
    StorageUnavailable,
    #[error("Saved data is invalid or from a newer version. It has not been overwritten.")]
    InvalidStorage,
    #[error("A protected file location was unavailable. No insecure storage fallback was used.")]
    UnsafePath,
    #[error("The operating system random generator was unavailable.")]
    RandomUnavailable,
    #[error("The core binary is missing or does not match the pinned digest and version.")]
    IncompatibleCore,
    #[error("The core returned an unsupported state format. No connectivity was inferred.")]
    UnsupportedCoreOutput,
    #[error("The verified networking helper is not installed in this engineering build. Saved networks remain available.")]
    HelperUnavailable,
    #[error("System networking permission was refused. No connection was started.")]
    PermissionDenied,
    #[error("The helper request was not authenticated or authorized.")]
    Unauthorized,
    #[error("The networking policy has not passed its release checks. Connection is disabled in this engineering build.")]
    ReleaseGate,
    #[error("The core stopped unexpectedly. Networking state must be reconciled.")]
    CoreFailed,
    #[error("The requested state transition is stale or invalid.")]
    InvalidTransition,
    #[error("Enter a peer virtual IPv4 address and a port between 1 and 65535.")]
    InvalidService,
}

pub type Result<T> = std::result::Result<T, Error>;
