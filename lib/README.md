Provides functions and types to get and set a Windows display profile.

A display [`Profile`] contains the resolution, frequency rate, scaling, rotation, etc. for
each active display.


# Cargo Feature

When the `dump` feature is active, each [Windows CCD API] function call also generates a `dump-*.json` file containing the inputs and outputs of the call.


[Windows CCD API]: https://learn.microsoft.com/en-us/windows-hardware/drivers/display/connecting-and-configuring-displays
