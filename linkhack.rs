// Some crumminess to make sure we link correctly

#[cfg(target_os = "linux")]
#[nolink]
native mod m { }

#[cfg(target_os = "macos")]
#[link_args = "-L. -lstdc++"]
#[nolink]
native mod m { }
