// Some crumminess to make sure we link correctly

#[cfg(target_os = "linux")]
#[nolink]
extern mod m { }

#[cfg(target_os = "macos")]
#[link_args = "-L. -lstdc++"]
#[nolink]
extern mod m { }
