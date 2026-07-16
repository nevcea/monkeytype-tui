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

#[cfg(test)]
mod tests {
    cycle_enum! {
        #[derive(Clone, Copy, PartialEq, Debug)]
        enum Tri { A = "a", B = "b", C = "c" }
        default = A;
    }

    #[test]
    fn label_returns_expected_strings() {
        assert_eq!(Tri::A.label(), "a");
        assert_eq!(Tri::B.label(), "b");
        assert_eq!(Tri::C.label(), "c");
    }

    #[test]
    fn next_cycles_forward_and_wraps() {
        assert_eq!(Tri::A.next(), Tri::B);
        assert_eq!(Tri::B.next(), Tri::C);
        assert_eq!(Tri::C.next(), Tri::A);
    }

    #[test]
    fn prev_cycles_backward_and_wraps() {
        assert_eq!(Tri::A.prev(), Tri::C);
        assert_eq!(Tri::C.prev(), Tri::B);
        assert_eq!(Tri::B.prev(), Tri::A);
    }

    #[test]
    fn default_uses_declared_variant() {
        assert_eq!(Tri::default(), Tri::A);
    }

    #[test]
    fn full_round_trip_returns_to_start() {
        for start in [Tri::A, Tri::B, Tri::C] {
            let mut v = start;
            for _ in 0..3 {
                v = v.next();
            }
            assert_eq!(v, start);

            let mut v = start;
            for _ in 0..3 {
                v = v.prev();
            }
            assert_eq!(v, start);
        }
    }
}
