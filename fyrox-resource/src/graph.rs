// Copyright (c) 2019-present Dmitry Stepanov and Fyrox Engine contributors.
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

//! Resource dependency graph. See [`ResourceDependencyGraph`] docs for more info.

use crate::{collect_used_resources, state::ResourceState, untyped::UntypedResource};
use fxhash::FxHashSet;

/// A node of [`ResourceDependencyGraph`].
pub struct ResourceGraphNode {
    /// A resource associated with the graph node.
    pub resource: UntypedResource,
    /// A list of children nodes of the graph.
    pub children: Vec<ResourceGraphNode>,
}

impl ResourceGraphNode {
    /// Creates a new resource graph node for a given untyped resource. This method is recursive -
    /// it will initialize the entire sub-graph of dependencies automatically.
    pub fn new(resource: &UntypedResource) -> Self {
        let mut children = Vec::new();

        // Look for dependent resources.
        let mut dependent_resources = FxHashSet::default();

        let header = resource.0.lock();
        if let ResourceState::Ok { ref data, .. } = header.state {
            (**data).as_reflect(&mut |entity| {
                collect_used_resources(entity, &mut dependent_resources);
            });
        }

        children.extend(
            dependent_resources
                .into_iter()
                .map(|r| ResourceGraphNode::new(&r)),
        );

        Self {
            resource: resource.clone(),
            children,
        }
    }

    /// Recursively prints the dependency graph node and its descendant nodes to the specified string, applying
    /// specified level offset.
    pub fn pretty_print(&self, level: usize, out: &mut String) {
        *out += &format!(
            "{}{}\n",
            String::from('\t').repeat(level),
            self.resource.kind()
        );

        for child in self.children.iter() {
            child.pretty_print(level + 1, out);
        }
    }

    /// Iterates over each dependency graph node and applying the specific function to them.
    pub fn for_each<F: FnMut(&UntypedResource)>(&self, func: &mut F) {
        func(&self.resource);

        for child in self.children.iter() {
            child.for_each(func)
        }
    }
}

/// Resource dependency graph allows you to collect all dependencies of a resource in structured form.
/// Internally, it uses reflection to look into resources content and find dependent resources. An example
/// of dependent resource is very simple: if you have a 3D model, then it most likely has a bunch of
/// textures - these textures are dependent resources. A more complex example - a game level could depend
/// on lots of prefabs, which in their turn may depend on other prefabs, textures, sounds, etc.
pub struct ResourceDependencyGraph {
    /// Root node of the graph.
    pub root: ResourceGraphNode,
}

impl ResourceDependencyGraph {
    /// Creates a new resource dependency graph starting from a given untyped resource.
    pub fn new(resource: &UntypedResource) -> Self {
        Self {
            root: ResourceGraphNode::new(resource),
        }
    }

    /// Iterates over each dependency graph node and applying the specific function to them.
    pub fn for_each<F: FnMut(&UntypedResource)>(&self, mut func: F) {
        self.root.for_each(&mut func)
    }

    /// Prints the entire dependency graph into a string.
    pub fn pretty_print(&self) -> String {
        let mut out = String::new();
        self.root.pretty_print(0, &mut out);
        out
    }
}
#[cfg(test)]
mod test {
    use super::*;
    use crate::untyped::ResourceKind;

    #[test]
    fn resource_graph_node_new() {
        let resource = UntypedResource::default();
        let node = ResourceGraphNode::new(&resource);

        assert_eq!(node.resource, resource);
        assert_eq!(node.children.len(), 0);
    }

    #[test]
    fn resource_graph_node_pretty_print() {
        let mut s = String::new();
        let mut node = ResourceGraphNode::new(&UntypedResource::new_pending(
            Default::default(),
            ResourceKind::External,
        ));
        let node2 = ResourceGraphNode::new(&UntypedResource::new_pending(
            Default::default(),
            ResourceKind::External,
        ));
        node.children.push(node2);
        node.pretty_print(1, &mut s);

        assert_eq!(s, "\tExternal\n\t\tExternal\n".to_string());
    }

    #[test]
    fn resource_dependency_graph_new() {
        let resource = UntypedResource::default();
        let graph = ResourceDependencyGraph::new(&resource);

        assert_eq!(graph.root.resource, resource);
        assert_eq!(graph.root.children.len(), 0);
    }

    #[test]
    fn resource_dependency_pretty_print() {
        let mut graph = ResourceDependencyGraph::new(&UntypedResource::new_pending(
            Default::default(),
            ResourceKind::External,
        ));
        graph
            .root
            .children
            .push(ResourceGraphNode::new(&UntypedResource::new_pending(
                Default::default(),
                ResourceKind::External,
            )));

        let s = graph.pretty_print();
        assert_eq!(s, "External\n\tExternal\n".to_string());
    }
}
