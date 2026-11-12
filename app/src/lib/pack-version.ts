/** The version an installed pack is compared against: the pack's own content fingerprint, so a
 *  release that leaves a dictionary unchanged does not make anyone download it again. Manifests
 *  from before per-pack versions fall back to the manifest-wide build version. */
export const packVersion = (p: { version?: string }, m: { version: string }): string => p.version ?? m.version;
