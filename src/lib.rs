#![deny(clippy::pedantic)]

use html5ever::tendril::TendrilSink;
use html5ever::{parse_document, ParseOpts};

/// Consumes a string that contains HTML5 tags and outputs a `String` containing the text content
/// inside the tags.
///
/// Basic usage:
///
/// ```rust
/// # use dissolve::strip_html_tags;
/// let input = "<html>Hello World!</html>";
/// let output = strip_html_tags(input);
/// assert_eq!(output, "Hello World!");
/// ```
#[must_use]
pub fn strip_html_tags(input: &str) -> String {
    parse_document(sink::TextOnly::default(), ParseOpts::default()).one(input)
}

mod sink {
    use std::borrow::Cow;
    use std::cell::RefCell;
    use std::rc::Rc;

    use html5ever::tendril::StrTendril;
    use html5ever::tree_builder::{ElementFlags, NodeOrText, QuirksMode, TreeSink};
    use html5ever::{Attribute, ExpandedName, QualName};

    #[derive(Default)]
    pub struct TextOnly {
        text: RefCell<String>,
    }

    pub struct Node {
        data: NodeData,
    }

    impl Node {
        fn new(data: NodeData) -> Rc<Self> {
            Rc::new(Self { data })
        }
    }

    enum NodeData {
        Document,
        Comment,
        ProcessingInstruction,
        Element { name: QualName },
    }

    type Handle = Rc<Node>;

    impl TreeSink for TextOnly {
        type Handle = Handle;
        type ElemName<'a> = ExpandedName<'a>;
        type Output = String;

        fn finish(self) -> Self::Output {
            self.text.into_inner()
        }

        fn parse_error(&self, _msg: Cow<'static, str>) {}

        fn get_document(&self) -> Self::Handle {
            Node::new(NodeData::Document)
        }

        fn elem_name<'a>(&'a self, target: &'a Self::Handle) -> Self::ElemName<'_> {
            match &target.data {
                NodeData::Element { name } => name.expanded(),
                _ => panic!("not an element!"),
            }
        }

        fn create_element(
            &self,
            name: QualName,
            _attrs: Vec<Attribute>,
            _flags: ElementFlags,
        ) -> Self::Handle {
            Node::new(NodeData::Element { name })
        }

        fn create_comment(&self, _text: StrTendril) -> Self::Handle {
            Node::new(NodeData::Comment)
        }

        fn create_pi(&self, _target: StrTendril, _data: StrTendril) -> Self::Handle {
            Node::new(NodeData::ProcessingInstruction)
        }

        fn append_doctype_to_document(
            &self,
            _name: StrTendril,
            _public_id: StrTendril,
            _system_id: StrTendril,
        ) {
        }

        fn append(&self, _parent: &Self::Handle, child: NodeOrText<Self::Handle>) {
            if let NodeOrText::AppendText(text) = &child {
                self.text.borrow_mut().push_str(text);
            }
        }

        fn append_based_on_parent_node(
            &self,
            _element: &Self::Handle,
            _prev_element: &Self::Handle,
            child: NodeOrText<Self::Handle>,
        ) {
            if let NodeOrText::AppendText(text) = &child {
                self.text.borrow_mut().push_str(text);
            }
        }

        fn append_before_sibling(
            &self,
            _sibling: &Self::Handle,
            _new_node: NodeOrText<Self::Handle>,
        ) {
            // This would be called for `InsertionPoint::BeforeSibling` but this enum variant is
            // currently not constructed in `html5ever`'s code.
            unimplemented!("Please fill an issue.")
        }

        fn get_template_contents(&self, _target: &Self::Handle) -> Self::Handle {
            Node::new(NodeData::Document)
        }

        fn same_node(&self, x: &Self::Handle, y: &Self::Handle) -> bool {
            Rc::ptr_eq(x, y)
        }

        fn set_quirks_mode(&self, _mode: QuirksMode) {}

        fn add_attrs_if_missing(&self, _target: &Self::Handle, _attrs: Vec<Attribute>) {}

        fn remove_from_parent(&self, _target: &Self::Handle) {}

        fn reparent_children(&self, _node: &Self::Handle, _new_parent: &Self::Handle) {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_strip_html_tag() {
        let input = "<html>Hello World!</html>";
        let output = strip_html_tags(input);
        assert_eq!(output, "Hello World!");
    }

    #[test]
    fn test_strip_nested_tags() {
        let input = "<html>Hello<div>World!</div></html>";
        let output = strip_html_tags(input);
        assert_eq!(output, "HelloWorld!");
    }

    #[test]
    fn test_preorder_traversal() {
        let input = "<html>Hel<div>lo</div>World!</html>";
        let output = strip_html_tags(input);
        assert_eq!(output, "HelloWorld!");
    }

    #[test]
    fn strip_template() {
        let input = r#"<html>aaa <template id="aaa">bbb </template><title>ccc ddd</title></html>"#;
        let output = strip_html_tags(input);
        assert_eq!(output, "aaa bbb ccc ddd");
    }

    #[test]
    fn strip_nested_a() {
        let input = r#"<html><a>a<a>b</a>c</a></html>"#;
        let output = strip_html_tags(input);
        assert_eq!(output, "abc");
    }

    #[test]
    fn strip_table() {
        let input = r#"<html>a<table> b<tr> <td>c</td> </tr>d </table>e</html>"#;
        let output = strip_html_tags(input);
        assert_eq!(output, "a b c d e");
    }

    #[test]
    fn malformed() {
        let input = r#"<html>a<b</html>"#;
        let output = strip_html_tags(input);
        assert_eq!(output, "a");

        let input = r#"<html>a < b</html>"#;
        let output = strip_html_tags(input);
        assert_eq!(output, "a < b");

        let input = r#"<html>a>b</html>"#;
        let output = strip_html_tags(input);
        assert_eq!(output, "a>b");

        let input = r#"<html>a > b</html>"#;
        let output = strip_html_tags(input);
        assert_eq!(output, "a > b");
    }
}
