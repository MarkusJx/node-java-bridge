#![allow(clippy::too_many_arguments, non_camel_case_types, dead_code)]

use std::os::raw::c_int;
use std::sync::Once;

#[cfg(windows)]
macro_rules! generate {
  (extern "C" {
      $(fn $name:ident($($param:ident: $ptype:ty$(,)?)*)$( -> $rtype:ty)?;)+
  }) => {
      struct Napi {
          $(
              $name: unsafe extern "C" fn(
                  $($param: $ptype,)*
              )$( -> $rtype)*,
          )*
      }

      #[inline(never)]
      fn panic_load<T>() -> T {
          panic!("Must load N-API bindings")
      }

      static mut NAPI: Napi = {
          $(
              unsafe extern "C" fn $name($(_: $ptype,)*)$( -> $rtype)* {
                  panic_load()
              }
          )*

          Napi {
              $(
                  $name,
              )*
          }
      };

      pub unsafe fn load(
          host: &libloading::Library,
      ) -> Result<(), libloading::Error> {
          NAPI = Napi {
              $(
                  $name: {
                    let symbol: Result<libloading::Symbol<unsafe extern "C" fn ($(_: $ptype,)*)$( -> $rtype)*>, libloading::Error> = host.get(stringify!($name).as_bytes());
                    match symbol {
                      Ok(f) => *f,
                      Err(e) => {
                        debug_assert!({
                          println!("Load Node-API [{}] from host runtime failed: {}", stringify!($name), e);
                          true
                        });
                        return Ok(());
                      }
                    }
                  },
              )*
          };

          Ok(())
      }

      $(
          #[inline]
          #[allow(clippy::missing_safety_doc)]
          pub unsafe fn $name($($param: $ptype,)*)$( -> $rtype)* {
              (NAPI.$name)($($param,)*)
          }
      )*
  };
}

#[cfg(not(windows))]
macro_rules! generate {
  (extern "C" {
    $(fn $name:ident($($param:ident: $ptype:ty$(,)?)*)$( -> $rtype:ty)?;)+
  }) => {
    extern "C" {
      $(
        pub fn $name($($param: $ptype,)*)$( -> $rtype)*;
      ) *
    }
  };
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub enum uv_run_mode {
    UV_RUN_DEFAULT = 0,
    UV_RUN_ONCE = 1,
    UV_RUN_NOWAIT = 2,
}

generate!(
    extern "C" {
        fn uv_run(loop_: *mut napi::sys::uv_loop_s, mode: uv_run_mode) -> c_int;
    }
);

static LOAD: Once = Once::new();

pub fn load_napi_library() {
    LOAD.call_once(|| {
        let host: libloading::Library = match libloading::os::windows::Library::this() {
            Ok(lib) => lib.into(),
            Err(err) => {
                panic!("Initialize libloading failed {}", err);
            }
        };

        if let Err(e) = unsafe { load(&host) } {
            panic!("Load N-API bindings failed {}", e);
        }
    });
}
