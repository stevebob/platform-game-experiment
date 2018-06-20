use aabb::Aabb;
use best::BestSetNonEmpty;
use cgmath::{Vector2, vec2};
use fnv::FnvHashMap;
use graphics;
use loose_quad_tree::LooseQuadTree;
use shape::AxisAlignedRect;

pub type EntityId = u32;

pub struct EntityCommon {
    top_left: Vector2<f32>,
    shape: AxisAlignedRect,
    colour: [f32; 3],
}

impl EntityCommon {
    fn new(top_left: Vector2<f32>, size: Vector2<f32>, colour: [f32; 3]) -> Self {
        Self {
            top_left,
            shape: AxisAlignedRect::new(size),
            colour,
        }
    }
    fn aabb(&self) -> Aabb {
        self.shape.aabb(self.top_left)
    }
    fn movement_aabb(&self, new_top_left: Vector2<f32>) -> Aabb {
        Aabb::union(&self.aabb(), &self.shape.aabb(new_top_left))
    }
}

pub type RendererUpdate = EntityCommon;

impl<'a> graphics::quad::Update for &'a RendererUpdate {
    fn size(&self) -> [f32; 2] {
        self.shape.dimensions().into()
    }
    fn position(&self) -> [f32; 2] {
        self.top_left.into()
    }
    fn colour(&self) -> [f32; 3] {
        self.colour
    }
}

#[derive(Default)]
struct EntityIdAllocator {
    next: u32,
}

impl EntityIdAllocator {
    fn allocate(&mut self) -> EntityId {
        let id = self.next;
        self.next += 1;
        id
    }
    fn reset(&mut self) {
        self.next = 0;
    }
}

pub struct GameState {
    player_id: Option<EntityId>,
    entity_id_allocator: EntityIdAllocator,
    common: FnvHashMap<EntityId, EntityCommon>,
    velocity: FnvHashMap<EntityId, Vector2<f32>>,
    static_aabb_quad_tree: LooseQuadTree<(Vector2<f32>, AxisAlignedRect)>,
}

impl GameState {
    pub fn new(size_hint: Vector2<f32>) -> Self {
        Self {
            player_id: None,
            entity_id_allocator: Default::default(),
            common: Default::default(),
            velocity: Default::default(),
            static_aabb_quad_tree: LooseQuadTree::new(size_hint),
        }
    }
    fn clear(&mut self) {
        self.player_id = None;
        self.entity_id_allocator.reset();
        self.common.clear();
        self.velocity.clear();
        self.static_aabb_quad_tree.clear();
    }
    fn add_static_solid(&mut self, common: EntityCommon) -> EntityId {
        let id = self.entity_id_allocator.allocate();
        self.static_aabb_quad_tree
            .insert(common.aabb(), (common.top_left, common.shape.clone()));
        self.common.insert(id, common);
        id
    }
    fn add_common(&mut self, common: EntityCommon) -> EntityId {
        let id = self.entity_id_allocator.allocate();
        self.common.insert(id, common);
        id
    }
    pub fn init_demo(&mut self) {
        self.clear();
        let player_id = self.add_common(EntityCommon::new(
            vec2(0., 0.),
            vec2(32., 64.),
            [1., 0., 0.],
        ));
        self.player_id = Some(player_id);
        self.velocity.insert(player_id, vec2(3., 7.));
        self.add_static_solid(EntityCommon::new(
            vec2(50., 200.),
            vec2(400., 20.),
            [1., 1., 0.],
        ));
        self.add_static_solid(EntityCommon::new(
            vec2(150., 250.),
            vec2(500., 20.),
            [1., 1., 0.],
        ));
        self.add_static_solid(EntityCommon::new(
            vec2(50., 450.),
            vec2(100., 20.),
            [1., 1., 0.],
        ));
        self.add_static_solid(EntityCommon::new(
            vec2(50., 500.),
            vec2(800., 20.),
            [1., 1., 0.],
        ));
    }
    pub fn update(&mut self) {
        for (id, velocity) in self.velocity.iter() {
            if let Some(common) = self.common.get_mut(id) {
                let movement = *velocity;
                let new_top_left = common.top_left + movement;
                let movement_aabb = common.movement_aabb(new_top_left);
                let mut movement_scale = BestSetNonEmpty::new(1.);
                self.static_aabb_quad_tree.for_each_intersection(
                    &movement_aabb,
                    |_solid_aabb, (solid_position, solid_shape)| {
                        let current_movement_scale =
                            common.shape.movement_vector_scale_after_collision(
                                common.top_left,
                                solid_shape,
                                *solid_position,
                                movement,
                            );
                        movement_scale.insert_lt(current_movement_scale);
                    },
                );
                let movement_scale = movement_scale.into_value();
                common.top_left += movement * movement_scale;
            }
        }
    }
    pub fn renderer_updates<'a>(&'a self) -> impl Iterator<Item = &'a RendererUpdate> {
        self.common.values()
    }
}
