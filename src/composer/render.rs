use serde::{ser::SerializeStruct, Serialize};

#[derive(Debug)]
pub struct Node<T> {
    pub idx: usize,
    pub value: T,
    pub parent: Option<usize>,
    pub children: Vec<usize>,
}

#[derive(Debug)]
pub struct Tree<T> {
    nodes: Vec<Node<T>>,
}

impl<T> Tree<T> {
    pub fn new() -> Tree<T> {
        Tree { nodes: vec![] }
    }

    pub fn root(&self) -> Option<&Node<T>> {
        self.get(0)
    }

    pub fn get(&self, idx: usize) -> Option<&Node<T>> {
        self.nodes.get(idx)
    }

    pub fn node_iter<'a>(&'a self, start: &'a Node<T>) -> NodeIter<T> {
        NodeIter {
            tree: self,
            idx_idx: 0,
            idxs: vec![&start.idx],
            skip: None,
        }
    }

    pub fn node_iter_with_skip<'a>(&'a self, start: &'a Node<T>, skip: Vec<usize>) -> NodeIter<T> {
        NodeIter {
            tree: self,
            idx_idx: 0,
            idxs: vec![&start.idx],
            skip: Some(skip),
        }
    }

    pub fn iter(&self) -> NodeIter<T> {
        match self.root() {
            Some(root) => self.node_iter(root),
            None => NodeIter {
                tree: self,
                idx_idx: 0,
                idxs: vec![],
                skip: None,
            },
        }
    }

    pub fn insert(&mut self, item: T, parent_idx: Option<usize>) -> usize {
        let new_idx = self.nodes.len();
        self.nodes.push(Node {
            idx: new_idx,
            parent: parent_idx,
            value: item,
            children: vec![],
        });

        if let Some(parent_idx) = parent_idx {
            self.nodes[parent_idx].children.push(new_idx)
        }

        new_idx
    }
}

impl<T> Default for Tree<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, T> IntoIterator for &'a Tree<T> {
    type Item = &'a Node<T>;

    type IntoIter = NodeIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct NodeIter<'a, T> {
    tree: &'a Tree<T>,
    idx_idx: usize,
    idxs: Vec<&'a usize>,
    skip: Option<Vec<usize>>,
}

impl<'a, T> Iterator for NodeIter<'a, T> {
    type Item = &'a Node<T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.idxs.get(self.idx_idx) {
            Some(idx) => {
                let ret = &self.tree.nodes[**idx];
                self.idxs.append(
                    &mut ret
                        .children
                        .iter()
                        .filter(|n| {
                            if let Some(skip) = &self.skip {
                                !skip.contains(n)
                            } else {
                                true
                            }
                        })
                        .collect(),
                );
                self.idx_idx += 1;

                Some(ret)
            }
            None => None,
        }
    }
}

impl<T> std::ops::Index<usize> for Tree<T> {
    type Output = Node<T>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.nodes[index]
    }
}

impl<T> std::ops::IndexMut<usize> for Tree<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.nodes[index]
    }
}

impl<T> Serialize for Tree<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        NodeWithTree {
            node: &self[0],
            tree: self,
        }
        .serialize(serializer)
    }
}

// Private serialization helper struct
struct NodeWithTree<'a, T> {
    pub node: &'a Node<T>,
    pub tree: &'a Tree<T>,
}

impl<'a, T> Serialize for NodeWithTree<'a, T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let num_fields = if self.node.children.is_empty() { 1 } else { 2 };
        let mut state = serializer.serialize_struct("Node", num_fields)?;

        state.serialize_field("segment", &self.node.value)?;

        if !self.node.children.is_empty() {
            state.serialize_field(
                "children",
                &self
                    .node
                    .children
                    .iter()
                    .map(|child_idx| NodeWithTree {
                        node: &self.tree[*child_idx],
                        tree: self.tree,
                    })
                    .collect::<Vec<_>>(),
            )?;
        }

        state.end()
    }
}
