# The Problem

Running [`windows-ccd-example`](../../../windows-ccd-example/src/main.rs) on my configuration
(two 1920 x 1080 monitors, one of them in vertical orientation) sets the desktop as a squared
1080 x 1080 area that is cloned in both monitors.

With this configuration, the `display-profile-cli` fails to apply any of [the saved profiles](../profiles/)
(except `AOC+Dell-cloned-squared`).

The files saved here are intented to help the investigation on why that fails.

The script [`commands.sh`](commands.sh) replicates the failure and collects the dump files:

```bash
# Succesfully applies the AOC+Dell-extended profile
./display-profile/data/cloned-square/commands.sh 1

# Runs windows-ccd-example
./display-profile/data/cloned-square/commands.sh 2

# Fails to apply the AOC+Dell-extended profile
./display-profile/data/cloned-square/commands.sh 3
```

# Solution
The issue was solved by including the desktop image info in the saved profile,
and using it when applying.

Now any profile can be applied after running `windows-ccd-example`.

There is still something to be investigated, because the profile `AOC+Dell-cloned-squared` results in a
non-squared desktop area when applied.
