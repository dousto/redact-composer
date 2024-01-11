use std::fmt::Debug;
use std::ops::Index;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug)]
/// An index based n-ary tree node.
pub struct Node<T> {
    /// Index of this node.
    pub idx: usize,
    /// Contained value of this node.
    pub value: T,
    /// This node's parent index as an [`Option`], or [`None`] if it is a root.
    pub parent: Option<usize>,
    /// The indices of this node's children.
    pub children: Vec<usize>,
}

#[derive(Debug)]
/// An n-ary index-based tree.
pub struct Tree<T> {
    nodes: Vec<Node<T>>,
}

impl<T> Tree<T> {
    /// Creates an empty tree.
    pub fn new() -> Tree<T> {
        Tree { nodes: vec![] }
    }

    /// The number of nodes in the tree.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns `true` iff the tree is empty.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Returns this tree's root, or [`None`] if it is empty.
    pub fn root(&self) -> Option<&Node<T>> {
        self.get(0)
    }

    /// Gets a node by it's index.
    pub fn get(&self, idx: usize) -> Option<&Node<T>> {
        self.nodes.get(idx)
    }

    /// Iterates a subtree starting from a given node. Order is not guaranteed.
    pub fn node_iter<'a>(&'a self, start: &'a Node<T>) -> NodeIter<T> {
        NodeIter {
            tree: self,
            idx_idx: 0,
            idxs: vec![&start.idx],
            skip: None,
        }
    }

    /// Iterates a subtree, skipping indices contained in `skip`.
    pub fn node_iter_with_skip<'a>(&'a self, start: &'a Node<T>, skip: Vec<usize>) -> NodeIter<T> {
        NodeIter {
            tree: self,
            idx_idx: 0,
            idxs: vec![&start.idx],
            skip: Some(skip),
        }
    }

    /// Returns a node iterator starting from the tree's root. Order is not guaranteed.
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

    /// Inserts a new value in this tree as a child of the `parent_idx` node.
    pub fn insert(&mut self, item: T, parent_idx: Option<usize>) -> usize {
        let new_idx = self.nodes.len();
        self.nodes.push(Node {
            idx: new_idx,
            parent: parent_idx,
            value: item,
            children: vec![],
        });

        if let Some(parent_idx) = parent_idx {
            self.nodes[parent_idx].children.push(new_idx);
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

/// A node iterator with optional `skip` indices.
#[derive(Debug)]
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

impl<Idx: std::slice::SliceIndex<[Node<T>]>, T> Index<Idx> for Tree<T> {
    type Output = Idx::Output;

    fn index(&self, index: Idx) -> &Self::Output {
        &self.nodes[index]
    }
}

impl<Idx: std::slice::SliceIndex<[Node<T>]>, T> std::ops::IndexMut<Idx> for Tree<T> {
    fn index_mut(&mut self, index: Idx) -> &mut Self::Output {
        &mut self.nodes[index]
    }
}

#[cfg(feature = "serde")]
impl<T> Serialize for Tree<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> crate::render::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        SerializeHelperNode::from((&self[0], self)).serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, T> Deserialize<'de> for Tree<T>
where
    T: Deserialize<'de> + Debug,
{
    fn deserialize<D>(deserializer: D) -> crate::render::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(DeserializeHelperNode::deserialize(deserializer)?.into())
    }
}

// Private serialization helper struct
#[cfg(feature = "serde")]
#[cfg_attr(feature = "serde", derive(Serialize))]
struct SerializeHelperNode<'a, T> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub val: &'a T,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Vec::is_empty"))]
    pub children: Vec<SerializeHelperNode<'a, T>>,
}

#[cfg(feature = "serde")]
#[cfg_attr(feature = "serde", derive(Deserialize))]
struct DeserializeHelperNode<T> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub val: T,
    #[cfg_attr(feature = "serde", serde(default = "Vec::new"))]
    pub children: Vec<DeserializeHelperNode<T>>,
}

#[cfg(feature = "serde")]
impl<'a, T> From<(&'a Node<T>, &'a Tree<T>)> for SerializeHelperNode<'a, T> {
    fn from(value: (&'a Node<T>, &'a Tree<T>)) -> SerializeHelperNode<'a, T> {
        let (node, tree) = value;
        SerializeHelperNode {
            val: &node.value,
            children: node
                .children
                .iter()
                .map(|n| SerializeHelperNode::from((&tree[*n], tree)))
                .collect::<Vec<_>>(),
        }
    }
}

#[cfg(feature = "serde")]
impl<T> From<DeserializeHelperNode<T>> for Tree<T> {
    fn from(value: DeserializeHelperNode<T>) -> Self {
        let mut nodes_to_add = vec![(0_usize, value, None)];
        let mut nodes = vec![];
        let mut id_counter = 1;

        while !nodes_to_add.is_empty() {
            let mut next_nodes = nodes_to_add
                .drain(..)
                .flat_map(|(idx, n, parent)| {
                    let (value, children) = (n.val, n.children);

                    let child_idx_range = id_counter..(id_counter + children.len());
                    id_counter += children.len();

                    nodes.push(Node {
                        idx,
                        value,
                        parent,
                        children: child_idx_range.clone().collect(),
                    });

                    child_idx_range
                        .zip(children.into_iter())
                        .map(move |(child_idx, child_node)| (child_idx, child_node, Some(idx)))
                })
                .collect::<Vec<_>>();

            nodes_to_add.append(&mut next_nodes);
        }

        Tree { nodes }
    }
}
