// Some crumminess to make sure we link correctly

#[cfg(target_os = "linux")]
#[link_args = "-lpthread -L. -ljs_static -lstdc++ -lz"]
#[nolink]
extern mod m { }

#[cfg(target_os = "macos")]
#[link_args = "-L. -ljs_static -lstdc++ -lz"]
#[nolink]
extern mod m { }
