Provides some convenience utilities for the [Windows CCD API] methods and types used via the [`windows`](https://crates.io/crates/windows) crate.


# Cargo Feature

When the `dump` feature is active, each [Windows CCD API] function call also generates a `dump-*.json` file containing the inputs and outputs of the call.


# References

* The [`windows`] crate
* Windows CCD API functions:
  * [QueryDisplayConfig]
  * [SetDisplayConfig]
  * [DisplayConfigGetDeviceInfo]


[Windows CCD API]: https://learn.microsoft.com/en-us/windows-hardware/drivers/display/connecting-and-configuring-displays
[`windows`]: https://microsoft.github.io/windows-docs-rs/doc/windows/index.html
[QueryDisplayConfig]: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-querydisplayconfig
[SetDisplayConfig]: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setdisplayconfig
[DisplayConfigGetDeviceInfo]: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-displayconfiggetdeviceinfo
