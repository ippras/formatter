//! https://stackoverflow.com/questions/59602202/how-can-i-retain-vector-elements-with-their-original-index

pub fn temp<T, F: FnMut(usize) -> bool>(mut f: F) -> impl FnMut(&T) -> bool {
    let mut index = 0;
    move |item| {
        if f(index) {
            index += 1;
            return true;
        }
        false
    }
}

pub fn index<T, F: FnMut(usize) -> bool>(mut f: F) -> impl FnMut(&T) -> bool {
    let mut index = 0;
    move |item| (f(index), index += 1).0
}

pub fn with_index<T, F: FnMut(usize, &T) -> bool>(mut f: F) -> impl FnMut(&T) -> bool {
    let mut index = 0;
    move |item| (f(index, item), index += 1).0
}
