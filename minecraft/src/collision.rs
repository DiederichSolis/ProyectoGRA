// Estructura que representa una caja de colisión con coordenadas mínimas y máximas en los ejes x, y, z.
#[derive(Debug, Clone)]
pub struct CollisionBox {
    pub min_x: f32, // Coordenada mínima en el eje X
    pub max_x: f32, // Coordenada máxima en el eje X
    pub min_y: f32, // Coordenada mínima en el eje Y
    pub max_y: f32, // Coordenada máxima en el eje Y
    pub min_z: f32, // Coordenada mínima en el eje Z
    pub max_z: f32, // Coordenada máxima en el eje Z
}

// Estructura que representa un punto de colisión con coordenadas x, y, z.
pub struct CollisionPoint {
    pub x: f32, // Coordenada X del punto de colisión
    pub y: f32, // Coordenada Y del punto de colisión
    pub z: f32, // Coordenada Z del punto de colisión
}

// Estructura que representa un rayo con un origen y una dirección.
pub struct Ray {
    pub origin: glam::Vec3,    // Origen del rayo
    pub direction: glam::Vec3, // Dirección del rayo
}

impl Ray {
    // Método que verifica si un rayo intersecta una caja de colisión.
    // Devuelve un vector de puntos de intersección si hay una colisión.
    pub fn intersects_box(&self, collision_box: &CollisionBox) -> Option<Vec<glam::Vec3>> {
        let mut tmin;
        let mut tmax;
        let tymin;
        let tymax;
        let tzmin;
        let tzmax;

        // Calculamos los inversos de las direcciones del rayo.
        let invdirx = 1.0 / self.direction.x;
        let invdiry = 1.0 / self.direction.y;
        let invdirz = 1.0 / self.direction.z;

        // Calculamos tmin y tmax para el eje X.
        if invdirx >= 0.0 {
            tmin = (collision_box.min_x - self.origin.x) * invdirx;
            tmax = (collision_box.max_x - self.origin.x) * invdirx;
        } else {
            tmin = (collision_box.max_x - self.origin.x) * invdirx;
            tmax = (collision_box.min_x - self.origin.x) * invdirx;
        }

        // Calculamos tymin y tymax para el eje Y.
        if invdiry >= 0.0 {
            tymin = (collision_box.min_y - self.origin.y) * invdiry;
            tymax = (collision_box.max_y - self.origin.y) * invdiry;
        } else {
            tymin = (collision_box.max_y - self.origin.y) * invdiry;
            tymax = (collision_box.min_y - self.origin.y) * invdiry;
        }

        // Verificamos si hay intersección en el eje X y Y.
        if tmin > tymax || tymin > tmax {
            return None; // No hay intersección.
        }
        if tymin > tmin {
            tmin = tymin; // Actualizamos tmin si es necesario.
        }
        if tymax < tmax {
            tmax = tymax; // Actualizamos tmax si es necesario.
        }

        // Calculamos tzmin y tzmax para el eje Z.
        if invdirz >= 0.0 {
            tzmin = (collision_box.min_z - self.origin.z) * invdirz;
            tzmax = (collision_box.max_z - self.origin.z) * invdirz;
        } else {
            tzmin = (collision_box.max_z - self.origin.z) * invdirz;
            tzmax = (collision_box.min_z - self.origin.z) * invdirz;
        }

        // Verificamos si hay intersección en el eje Z y en el tiempo t.
        if tmin > tzmax || tzmin > tmax || tmin < 0.0 || tmax < 0.0 {
            return None; // No hay intersección.
        }

        if tzmin > tmin {
            tmin = tzmin; // Actualizamos tmin si es necesario.
        }
        if tzmax < tmax {
            tmax = tzmax; // Actualizamos tmax si es necesario.
        }

        // Devolvemos los puntos de intersección.
        Some(vec![
            self.origin + self.direction * tmin, // Punto de entrada
            self.origin + self.direction * tmax, // Punto de salida
        ])
    }
}

// Estructura que representa el resultado de una intersección de rayo.
#[derive(Debug)]
pub struct RayResult {
    pub points: Vec<glam::Vec3>, // Puntos de intersección
    pub collision: CollisionBox,   // Caja de colisión correspondiente
}

impl CollisionPoint {
    // Constructor para crear un nuevo CollisionPoint.
    pub fn new(x: f32, y: f32, z: f32) -> CollisionPoint {
        CollisionPoint { x, y, z }
    }
}

impl CollisionBox {
    // Método que devuelve el centro de la caja de colisión.
    pub fn center(&self) -> glam::Vec3 {
        glam::vec3(
            self.min_x + (self.max_x - self.min_x) / 2.0,
            self.min_y + (self.max_y - self.min_y) / 2.0,
            self.min_z + (self.max_z - self.min_z) / 2.0,
        )
    }

    // Método para crear una CollisionBox a partir de la posición de un bloque.
    pub fn from_block_position(x: f32, y: f32, z: f32) -> Self {
        CollisionBox {
            min_x: x,
            max_x: x + 1.0,
            min_y: y,
            max_y: y + 1.0,
            min_z: z,
            max_z: z + 1.0,
        }
    }

    // Método que devuelve la posición de un bloque a partir de la caja de colisión.
    pub fn to_block_position(&self) -> glam::Vec3 {
        glam::vec3(self.min_x, self.min_y, self.min_z)
    }

    // Constructor para crear una nueva CollisionBox con dimensiones específicas.
    pub fn new(x: f32, y: f32, z: f32, width: f32, height: f32, depth: f32) -> CollisionBox {
        CollisionBox {
            min_x: x,
            max_x: x + width,
            min_y: y,
            max_y: y + height,
            min_z: z,
            max_z: z + depth,
        }
    }

    // Método que verifica si un CollisionPoint intersecta con la caja de colisión.
    pub fn intersects_point(&self, point: &CollisionPoint) -> bool {
        point.x >= self.min_x
            && point.x <= self.max_x
            && point.y >= self.min_y
            && point.y <= self.max_y
            && point.z >= self.min_z
            && point.z <= self.max_z
    }

    // Método que verifica si esta caja de colisión intersecta con otra.
    pub fn intersects(&self, other: &CollisionBox) -> bool {
        self.min_x <= other.max_x
            && self.max_x >= other.min_x
            && self.min_y <= other.max_y
            && self.max_y >= other.min_y
            && self.min_z <= other.max_z
            && self.max_z >= other.min_z
    }

    // Método de intersección de dirección (pendiente de implementación).
    pub fn intersects_direction() {
        todo!()
    }
}

// Implementación de la operación de suma para CollisionBox con glam::Vec3.
impl std::ops::Add<glam::Vec3> for CollisionBox {
    type Output = CollisionBox;

    fn add(self, rhs: glam::Vec3) -> Self::Output {
        CollisionBox::new(
            self.min_x + rhs.x,
            self.min_y + rhs.y,
            self.min_z + rhs.z,
            self.max_x - self.min_x,
            self.max_y - self.min_y,
            self.max_z - self.min_z,
        )
    }
}
