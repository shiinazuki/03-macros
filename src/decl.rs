#[macro_export]
macro_rules! my_vec {
    () => {
        ::std::vec::Vec::new()
    };

    ($elem:expr; $count:expr) => {
        ::std::iter::repeat_n($elem, $count).collect::<::std::vec::Vec<_>>()
    };

    ($($x:expr),+ $(,)?) => {
        ::std::vec::Vec::from([$($x),+])
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn list_form() {
        let v = my_vec![1, 2, 3];
        assert_eq!(v, [1, 2, 3]);
    }

    #[test]
    fn works_with_expressions() {
        let v = my_vec![1 + 1, 2 * 2];
        assert_eq!(v, [2, 4]);
    }

    #[test]
    fn caller_defines_its_own_vec() {
        struct Vec; // 调用方自己有个叫 Vec 的类型
        let _ = Vec;
        let v = my_vec![1, 2, 3];
        assert_eq!(v, [1, 2, 3]);
    }

    #[test]
    fn empty_form() {
        let v: Vec<i32> = my_vec![];
        assert!(v.is_empty());
    }

    #[test]
    fn repeat_form() {
        let v = my_vec![7; 4];
        assert_eq!(v, [7, 7, 7, 7]);
    }

    #[test]
    fn trailing_comma() {
        assert_eq!(my_vec![1, 2, 3,], [1, 2, 3]);
    }

    #[test]
    fn single_element() {
        assert_eq!(my_vec![5], [5]);
    }
}
