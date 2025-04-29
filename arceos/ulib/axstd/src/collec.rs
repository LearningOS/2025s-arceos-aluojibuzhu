// axstd/src/collec.rs
#![cfg(feature = "alloc")]  // 确保只在启用 alloc 时编译
extern crate alloc;

use alloc::vec::Vec;
use alloc::boxed::Box;

use ahash::AHasher;
use core::hash::{BuildHasher, BuildHasherDefault, Hash, Hasher};
use core::iter::Iterator;
// 新增迭代器相关实现
pub struct Iter<'a, K, V> {
    map: &'a HashMap<K, V>,
    bucket_idx: usize,
    current_node: Option<&'a Node<K, V>>,
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // 如果当前节点存在，返回它的键值对并移动到下一个节点
            if let Some(node) = self.current_node {
                let result = (&node.key, &node.value);
                self.current_node = node.next.as_deref();
                return Some(result);
            }
            
            // 否则移动到下一个桶
            if self.bucket_idx >= self.map.buckets.len() {
                return None;
            }
            
            self.current_node = self.map.buckets[self.bucket_idx].as_deref();
            self.bucket_idx += 1;
        }
    }
}

type DefaultHasher = BuildHasherDefault<AHasher>;
// 链表节点
struct Node<K, V> {
    key: K,
    value: V,
    next: Option<Box<Node<K, V>>>,
}

// HashMap 结构
pub struct HashMap<K, V> {
    buckets: Vec<Option<Box<Node<K, V>>>>,
    len: usize,
}

impl<K, V> HashMap<K, V>
where
    K: Hash + Eq,
{
    pub fn iter(&self) -> Iter<K, V> {
        Iter {
            map: self,
            bucket_idx: 0,
            current_node: None,
        }
    }
    // 初始化（默认容量 8）
    pub fn new() -> Self {
        const INITIAL_CAPACITY: usize = 8;
        Self {
            buckets: (0..INITIAL_CAPACITY).map(|_| None).collect(),
            len: 0,
        }
    }

    // 计算 key 的哈希值（简单取模）
    fn hash(&self, key: &K) -> usize {
        let mut hasher = DefaultHasher::default().build_hasher();
        key.hash(&mut hasher);
        (hasher.finish() as usize) % self.buckets.len()
    }
    // 插入键值对
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if self.len >= self.buckets.len() {
            self.resize();
        }

        let idx = self.hash(&key);
        let mut node = &mut self.buckets[idx];

        // 遍历链表，检查是否已存在 key
        while let Some(n) = node {
            if n.key == key {
                let old_value = core::mem::replace(&mut n.value, value);
                return Some(old_value);
            }
            node = &mut n.next;
        }

        // 插入新节点到链表头部
        *node = Some(Box::new(Node {
            key,
            value,
            next: None,
        }));
        self.len += 1;
        None
    }

    // 查询键值
    pub fn get(&self, key: &K) -> Option<&V> {
        let idx = self.hash(key);
        let mut node = &self.buckets[idx];

        while let Some(n) = node {
            if &n.key == key {
                return Some(&n.value);
            }
            node = &n.next;
        }
        None
    }

    // 删除键值
    pub fn remove(&mut self, key: &K) -> Option<V> {
        let idx = self.hash(key);
        let head = &mut self.buckets[idx];

        // 处理头节点
        if let Some(n) = head {
            if &n.key == key {
                let node = head.take().unwrap();
                *head = node.next;
                self.len -= 1;
                return Some(node.value);
            }
        }

        // 遍历后续节点
        let mut prev = head;
        while let Some(curr) = prev {
            if let Some(next) = &mut curr.next {
                if &next.key == key {
                    let node = curr.next.take().unwrap();
                    curr.next = node.next;
                    self.len -= 1;
                    return Some(node.value);
                }
            }
            prev = &mut curr.next;
        }
        None
    }

    // 动态扩容（2 倍增长）
    fn resize(&mut self) {
        let new_capacity = self.buckets.len() * 2;
        let mut new_buckets = (0..new_capacity).map(|_| None).collect::<Vec<_>>();

        for bucket in self.buckets.drain(..) {
            let mut node = bucket;
            while let Some(mut n) = node {
                node = n.next.take(); // 断开链表
                let idx = {
                    let mut hasher =DefaultHasher::default().build_hasher();
                    n.key.hash(&mut hasher);
                    (hasher.finish() as usize) % new_capacity
                };
                n.next = new_buckets[idx].take();
                new_buckets[idx] = Some(n);
            }
        }
        self.buckets = new_buckets;
    }
}