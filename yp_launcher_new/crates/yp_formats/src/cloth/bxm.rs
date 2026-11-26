use glam::Vec3;

#[derive(Clone, Copy)]
pub(super) struct Node<'a> {
    nodes: &'a [u8],
    data: &'a [u8],
    strings: &'a [u8],
    index: usize,
}

fn short(bytes: &[u8], at: usize) -> Option<usize> {
    Some(u16::from_be_bytes(bytes.get(at..at + 2)?.try_into().ok()?) as usize)
}

impl<'a> Node<'a> {
    pub fn root(bytes: &'a [u8], expected: &str) -> Option<Self> {
        if !matches!(bytes.get(..4)?, b"BXM\0" | b"XML\0") {
            return None;
        }
        let count = short(bytes, 8)?;
        if count == 0 {
            return None;
        }
        let nodes_end = 16 + count * 8;
        let data_count = short(bytes, 10)?;
        let data_end = nodes_end + data_count * 4;
        let size = u32::from_be_bytes(bytes.get(12..16)?.try_into().ok()?) as usize;
        let tree = Self {
            nodes: bytes.get(16..nodes_end)?,
            data: bytes.get(nodes_end..data_end)?,
            strings: bytes.get(data_end..data_end.checked_add(size)?)?,
            index: 0,
        };
        if tree.strings.last() != Some(&0) {
            return None;
        }
        let mut visited = vec![false; count];
        visited[0] = true;
        for index in 0..count {
            if !visited[index] {
                return None;
            }
            let at = index * 8;
            let children = short(tree.nodes, at)?;
            let first = short(tree.nodes, at + 2)?;
            let attributes = short(tree.nodes, at + 4)?;
            let pair = short(tree.nodes, at + 6)?;
            if pair + attributes >= data_count {
                return None;
            }
            if children > 0 {
                if first <= index {
                    return None;
                }
                let range = visited.get_mut(first..first + children)?;
                if range.contains(&true) {
                    return None;
                }
                range.fill(true);
            }
        }
        for pair in tree.data.chunks_exact(4) {
            let name = short(pair, 0)?;
            let value = short(pair, 2)?;
            if name >= size || (value != usize::from(u16::MAX) && value >= size) {
                return None;
            }
        }
        (tree.name()? == expected).then_some(tree)
    }

    fn field(self, offset: usize) -> Option<&'a str> {
        let pair = short(self.nodes, self.index * 8 + 6)?;
        let start = short(self.data, pair * 4 + offset)?;
        if start == usize::from(u16::MAX) {
            return None;
        }
        let bytes = self.strings.get(start..)?;
        let end = bytes.iter().position(|&c| c == 0)?;
        std::str::from_utf8(&bytes[..end]).ok()
    }

    pub fn name(self) -> Option<&'a str> {
        self.field(0)
    }

    pub fn children(self) -> impl ExactSizeIterator<Item = Self> {
        let at = self.index * 8;
        let first = short(self.nodes, at + 2).unwrap_or(0);
        let end = first + short(self.nodes, at).unwrap_or(0);
        (first..end).map(move |index| Self { index, ..self })
    }

    pub fn child(self, name: &str) -> Option<Self> {
        self.children().find(|n| n.name() == Some(name))
    }

    pub fn integer(self, name: &str, default: i32) -> Option<i32> {
        self.child(name)
            .map_or(Some(default), |n| n.field(2)?.trim().parse().ok())
    }

    pub fn bone(self, name: &str) -> Option<Option<i16>> {
        let Some(n) = self.child(name) else {
            return Some(None);
        };
        match n.field(2)?.trim().parse::<i16>().ok()? {
            -2 | -1 | 4095 => Some(None),
            v @ 0..=4094 => Some(Some(v)),
            _ => None,
        }
    }

    pub fn reference(self, name: &str, flag: u16) -> Option<(Option<i16>, bool)> {
        let v = self.integer(name, 4095)?;
        if matches!(v, -2 | -1) {
            return Some((None, false));
        }
        if !(-32768..=65535).contains(&v) {
            return None;
        }
        let raw = v as u16;
        if raw & !(0xfff | flag) != 0 {
            return None;
        }
        let bone = (raw & 0xfff) as i16;
        Some(((bone != 4095).then_some(bone), raw & flag != 0))
    }

    pub fn scalar(self, name: &str, default: f32) -> Option<f32> {
        let Some(n) = self.child(name) else {
            return Some(default);
        };
        n.field(2)?
            .trim()
            .parse::<f32>()
            .ok()
            .filter(|v| v.is_finite())
    }

    pub fn count(self, name: &str) -> Option<usize> {
        self.child(name)?.field(2)?.trim().parse().ok()
    }

    pub fn vector(self, name: &str, default: Vec3) -> Option<Vec3> {
        let Some(n) = self.child(name) else {
            return Some(default);
        };
        let mut parts = n.field(2)?.split_ascii_whitespace();
        let v = Vec3::new(
            parts.next()?.parse().ok()?,
            parts.next()?.parse().ok()?,
            parts.next()?.parse().ok()?,
        );
        (parts.next().is_none() && v.length_squared().is_finite()).then_some(v)
    }
}
