//! Shared declarative macros.

/// Define a small enum that cycles through its variants via `next()`/`prev()`
/// and exposes a static `label()`. Removes the near-identical `next`/`prev`/
/// `label` match triples repeated across settings enums.
///
/// ```ignore
/// cycle_enum! {
///     #[derive(Clone, Copy, PartialEq)]
///     pub enum Foo { A = "a", B = "b" }
///     default = A;
/// }
/// ```
macro_rules! cycle_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $( $variant:ident = $label:literal ),+ $(,)?
        }
        $( default = $default:ident; )?
    ) => {
        $(#[$meta])*
        $vis enum $name {
            $( $variant ),+
        }

        impl $name {
            pub fn label(self) -> &'static str {
                match self { $( Self::$variant => $label ),+ }
            }

            pub fn next(self) -> Self {
                const ORDER: &[$name] = &[ $( $name::$variant ),+ ];
                let i = ORDER.iter().position(|&v| v == self).unwrap_or(0);
                ORDER[(i + 1) % ORDER.len()]
            }

            pub fn prev(self) -> Self {
                const ORDER: &[$name] = &[ $( $name::$variant ),+ ];
                let i = ORDER.iter().position(|&v| v == self).unwrap_or(0);
                ORDER[(i + ORDER.len() - 1) % ORDER.len()]
            }
        }

        $(
            impl Default for $name {
                fn default() -> Self { Self::$default }
            }
        )?
    };
}
