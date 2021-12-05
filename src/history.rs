use std::cell::RefCell;
use std::rc::{Rc, Weak};


// Tree struct based on https://doc.rust-lang.org/stable/book/ch15-06-reference-cycles.html
pub struct HistoryTreeEntry {
    label: String,
    parent: Weak<Self>, // option is not needed (I guess), we can just use a null weak to represent the root
    last_visited_child: RefCell<Option<Weak<Self>>>,
    children: RefCell<Vec<Rc<Self>>>,
}

impl HistoryTreeEntry {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            parent: Weak::new(),
            children: RefCell::new(vec![]),
            last_visited_child: RefCell::new(None),
        }
    }

    /// Convenience method for accessing the name of the last visited child, if it exists.
    /// Returns an owned String, because I couldn't figure out the borrowing here.
    pub fn last_visited_child_label(&self) -> Option<String> {
        self.last_visited_child
            .borrow()
            .as_ref()
            .and_then(|ptr| ptr.upgrade())
            // note: tried .map(|parent| parent.label.as_str()), but it's no good.
            .map(|parent| parent.label.clone())
    }
}

struct HistoryTree {
    root: Rc<HistoryTreeEntry>,
    current_entry: Rc<HistoryTreeEntry>,
}

impl HistoryTree {

    pub fn current_entry(&self) -> &Rc<HistoryTreeEntry> {
        &self.current_entry
    }

    pub fn visit(&mut self, fname: &str) {
        let found_child = self.current_entry.children.borrow().iter()
            .find(|child| child.label == fname).map(|c| c.clone());

        let child = found_child.unwrap_or_else(|| {
            // no such child found, create a new one
            let mut child = HistoryTreeEntry::new(fname);
            child.parent = Rc::downgrade(&self.current_entry);

            let child = Rc::new(child);
            self.current_entry.children.borrow_mut().push(Rc::clone(&child));
            child
        });

        self.current_entry.last_visited_child.replace(Some(Rc::downgrade(&child)));
        self.current_entry = child;
    }

    pub fn go_up(&mut self) {
        if let Some(parent) = self.current_entry.parent.upgrade() {
            self.current_entry = Rc::clone(&parent);
        } // if the parent is None, we're at the root, so no need to do anything
    }

    pub fn go_to_root(&mut self) {
        self.current_entry = Rc::clone(&self.root);
    }

}

#[cfg(test)]
mod tests_for_history_tree {
    use super::*;

    fn init_history_tree() -> HistoryTree {
        let root = Rc::new(HistoryTreeEntry {
            label: "/".to_string(),
            parent: Weak::new(),
            last_visited_child: RefCell::new(None),
            children: RefCell::new(vec![]),
        });

        HistoryTree {
            root: Rc::clone(&root),
            current_entry: root,
        }
    }

    #[test]
    fn test_history_tree_visit() {
        let mut tree = init_history_tree();

        tree.visit("foo");
        assert_eq!(tree.current_entry().label, "foo");
        assert_eq!(tree.current_entry().parent.upgrade().unwrap().label, "/");

        tree.visit("bar");
        assert_eq!(tree.current_entry().label, "bar");
        assert_eq!(tree.current_entry().parent.upgrade().unwrap().label, "foo");
        assert_eq!(tree.current_entry().parent.upgrade().unwrap().parent.upgrade().unwrap().label, "/");

    }

    #[test]
    fn test_history_tree_go_up_down() {
        let mut tree = init_history_tree();

        tree.visit("foo");
        tree.visit("bar");

        tree.go_up();
        assert_eq!(tree.current_entry().label, "foo");
        assert_eq!(tree.current_entry().children.borrow()[0].label, "bar");

        tree.go_up();
        assert_eq!(tree.current_entry().label, "/");
        assert_eq!(tree.current_entry().children.borrow()[0].label, "foo");

        tree.go_up();
        assert_eq!(tree.current_entry().label, "/");
        assert_eq!(tree.current_entry().children.borrow()[0].label, "foo");

    }

    #[test]
    fn test_tree_pointer_counts() {
        let mut tree = init_history_tree();
        tree.visit("foo");
        let foo = Rc::downgrade(&tree.current_entry());
        tree.visit("bar");
        let bar = Rc::downgrade(&tree.current_entry());

        assert_eq!(Rc::weak_count(&tree.root), 1); // the child (foo)

        assert_eq!(Weak::strong_count(&foo), 1); // the root
        assert_eq!(Weak::weak_count(&foo), 3); // the child, last_visited_child of the root and the variable 'foo' above

        assert_eq!(Weak::strong_count(&bar), 2); // the parent (foo) and the tree current entry
        assert_eq!(Weak::weak_count(&bar), 2); // the variable 'bar' above, and last_visited_child of foo

        tree.go_up(); tree.go_up();
        assert_eq!(Weak::strong_count(&bar), 1); // the parent only now
        assert_eq!(Weak::weak_count(&bar), 2); // the variable 'bar' above, and last_visited_child of foo

        tree.visit("baz");
        assert_eq!(Rc::weak_count(&tree.root), 2); // two children

    }

    #[test]
    fn test_last_visisted_child() {
        let mut tree = init_history_tree();
        tree.visit("foo");
        let foo = Rc::clone(tree.current_entry());
        tree.go_up();
        assert!(Rc::ptr_eq(&foo, &tree.current_entry().last_visited_child.borrow().as_ref().unwrap().upgrade().unwrap()));
    }

    #[test]
    fn test_go_to_root() {
        let mut tree = init_history_tree();
        let root = Rc::clone(tree.current_entry());
        tree.visit("foo");
        tree.visit("bar");
        tree.visit("baz");
        tree.go_to_root();
        assert!(Rc::ptr_eq(&root, tree.current_entry()));
    }

}
