use cgmath::{Angle, Vector3};
use log::info;

use crate::world::node::{IsNode, NodeRange, PhysNode};

use super::ext_adj::ExtAdjacency;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub enum Topology {
    Net,
    Ring,
    Line,
    Star,
}

impl Topology {
    const STEP: f32 = 6.0;

    pub(super) fn positions(self, origin: Vector3<f32>, count: usize) -> Vec<Vector3<f32>> {
        let origin = Vector3::new(origin.x, origin.y, 0.0);

        match self {
            Self::Net => Self::positions_net(origin, count),
            Self::Star => Self::positions_star(origin, count),
            Self::Line => Self::positions_line(origin, count),
            Self::Ring => Self::positions_ring(origin, count),
        }
    }

    fn positions_net(origin: Vector3<f32>, count: usize) -> Vec<Vector3<f32>> {
        let width = (count as f64).sqrt().ceil() as usize;
        let mut positions = vec![(0.0, 0.0, 0.0).into(); count];

        let mut row = 0;
        let mut column = 0;

        for (i, pos) in positions.iter_mut().enumerate() {
            if i != 0 && i % width == 0 {
                row += 1;
                column = 0;
            }

            let x = column as f32 * Self::STEP;
            let y = row as f32 * Self::STEP;

            *pos = (origin.x + x, origin.y + y, 0.0).into();
            column += 1;
        }

        positions
    }

    fn positions_star(origin: Vector3<f32>, count: usize) -> Vec<Vector3<f32>> {
        use cgmath::Deg;
        const RADIUS: f32 = 10.0;
        let deg_off = 360.0 / count as f32;

        let mut positions = vec![(0.0, 0.0, 0.0).into(); count];
        positions[0] = origin;

        let mut i = 0.0;
        for pos in positions.iter_mut().skip(1) {
            let x = Deg(i * deg_off).cos();
            let y = Deg(i * deg_off).sin();

            *pos = (RADIUS * x + origin.x, RADIUS * y + origin.y, 0.0).into();

            i += 1.0;
        }

        positions
    }

    fn positions_line(origin: Vector3<f32>, count: usize) -> Vec<Vector3<f32>> {
        let mut positions = vec![(0.0, 0.0, 0.0).into(); count];

        let mut start = origin;
        for pos in positions.iter_mut() {
            *pos = start;
            start.x += Self::STEP;
        }

        positions
    }

    fn positions_ring(origin: Vector3<f32>, count: usize) -> Vec<Vector3<f32>> {
        let mut positions = vec![];
        let width = (count as f32 / 4.0).ceil() as usize;

        let mut idx = 0;
        let mut x_scale = 1;
        let mut y_scale = 0;

        let mut pos = origin;
        info!("count = {count}");
        while idx < count {
            if idx != 0 && idx % width == 0 {
                let temp = y_scale;
                y_scale = -x_scale;
                x_scale = temp;
            }

            pos.x += x_scale as f32 * Self::STEP;
            pos.y += y_scale as f32 * Self::STEP;
            positions.push(pos);

            idx += 1;
        }

        positions
    }

    pub(super) fn connect(self, adjacency: &mut ExtAdjacency, nodes: NodeRange<PhysNode>) {
        match self {
            Self::Net => Self::connect_net(adjacency, nodes),
            Self::Star => Self::connect_star(adjacency, nodes),
            Self::Line => Self::connect_line(adjacency, nodes),
            Self::Ring => Self::connect_ring(adjacency, nodes),
        }
    }

    fn connect_net(adjacency: &mut ExtAdjacency, nodes: NodeRange<PhysNode>) {
        let start = nodes.start().idx();

        for i in nodes.iter() {
            for j in nodes.iter().skip((i.idx() - start) as usize + 1) {
                adjacency.set(i, j);
                adjacency.set(j, i);
            }
        }
    }

    fn connect_star(adjacency: &mut ExtAdjacency, nodes: NodeRange<PhysNode>) {
        let first = nodes.start();
        for i in nodes.iter().skip(1) {
            adjacency.set(i, first);
            adjacency.set(first, i);
        }
    }

    fn connect_line(adjacency: &mut ExtAdjacency, nodes: NodeRange<PhysNode>) {
        for i in nodes.iter().skip(1) {
            let j = unsafe { PhysNode::new(i.idx() - 1) };
            adjacency.set(i, j);
            adjacency.set(j, i);
        }
    }

    fn connect_ring(adjacency: &mut ExtAdjacency, nodes: NodeRange<PhysNode>) {
        for i in nodes.iter().skip(1) {
            let j = unsafe { PhysNode::new(i.idx() - 1) };
            adjacency.set(i, j);
            adjacency.set(j, i);
        }

        let start = nodes.start();
        let end = nodes.end();

        adjacency.set(start, end);
        adjacency.set(end, start);
    }
}
