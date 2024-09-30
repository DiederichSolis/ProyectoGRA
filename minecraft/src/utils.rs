use crate::world::CHUNK_SIZE;
use glam::{vec3, Vec3};

/// Módulo que contiene utilidades matemáticas.
pub(crate) mod math_utils {
    /// Estructura `Plane` que representa un plano en el espacio tridimensional.
    #[derive(Debug)]
    pub struct Plane {
        /// Un punto que pertenece al plano.
        pub point: glam::Vec3,
        /// El vector normal al plano.
        pub normal: glam::Vec3,
    }

    impl Plane {
        /// Calcula la distancia firmada de un punto dado al plano.
        ///
        /// # Parámetros:
        /// - `point`: El punto cuya distancia al plano se desea calcular.
        ///
        /// # Retorno:
        /// La distancia firmada del punto al plano.
        pub fn signed_plane_dist(&self, point: glam::Vec3) -> f32 {
            (point - self.point).dot(self.normal)
        }
    }
}

/// Módulo que contiene las funciones relacionadas con ruido (noise).
pub(crate) mod noise {
    use std::fmt::Debug;
    use crate::world::RNG_SEED;
    use glam::Vec2;

    const WRAP: u32 = 256;

    /// Tabla de permutación utilizada para generar ruido.
    lazy_static! {
        pub static ref PERM_TABLE: Vec<u32> = {
            let mut table: Vec<u32> = (0..WRAP).collect();
            shuffle(&mut table);
            for i in 0..WRAP {
                table.push(table[i as usize]);
            }
            table
        };
    }

    /// Función que mezcla los elementos de un vector.
    ///
    /// # Parámetros:
    /// - `vec`: El vector a mezclar.
    ///
    /// # Retorno:
    /// El vector mezclado.
    pub fn shuffle<T: Copy + Debug>(vec: &mut Vec<T>) -> &mut Vec<T> {
        use rand::prelude::*;
        let mut rng = StdRng::seed_from_u64(RNG_SEED);

        for i in (0..vec.len()).rev() {
            let a: usize = if i > 0 {
                f32::max(f32::floor(rng.gen::<f32>() * (i - 1) as f32), 0.0) as usize
            } else {
                0
            };
            vec.swap(i, a);
        }
        vec
    }

    /// Obtiene las constantes de las esquinas en base a un valor dado.
    ///
    /// # Parámetros:
    /// - `v`: El valor sobre el cual se realiza la operación.
    ///
    /// # Retorno:
    /// Un vector 2D con las constantes correspondientes.
    fn get_corner_consts(v: u32) -> Vec2 {
        let h = v & 3;
        match h {
            0 => glam::vec2(1.0, 1.0),
            1 => glam::vec2(-1.0, 1.0),
            2 => glam::vec2(-1.0, -1.0),
            _ => glam::vec2(1.0, -1.0),
        }
    }

    /// Genera ruido Perlin.
    ///
    /// # Parámetros:
    /// - `x`: La coordenada `x`.
    /// - `y`: La coordenada `y`.
    /// - `per`: Un valor de permutación.
    ///
    /// # Retorno:
    /// El valor de ruido Perlin calculado.
    pub fn perlin_noise(x: f32, y: f32, per: u32) -> f32 {
        let int_x = f32::floor(x) as u32;
        let int_y = f32::floor(y) as u32;

        let surflet = |grid_x: u32, grid_y: u32| {
            let dist_x = f32::abs(x - grid_x as f32) % WRAP as f32;
            let dist_y = f32::abs(y - grid_y as f32) % WRAP as f32;
            let poly_x = 1.0 - 6.0 * f32::powi(dist_x, 5) + 15.0 * f32::powi(dist_x, 4)
                - 10.0 * f32::powi(dist_x, 3);
            let poly_y = 1.0 - 6.0 * f32::powi(dist_y, 5) + 15.0 * f32::powi(dist_y, 4)
                - 10.0 * f32::powi(dist_y, 3);
            let hashed = PERM_TABLE[(PERM_TABLE[(grid_x % per) as usize] + (grid_y % per)) as usize];
            let grad = (x - grid_x as f32) * get_corner_consts(hashed).x
                + (y - grid_y as f32) * get_corner_consts(hashed).y;
            poly_x * poly_y * grad
        };

        f32::clamp(
            surflet(int_x, int_y)
                + surflet(int_x + 1, int_y)
                + surflet(int_x, int_y + 1)
                + surflet(int_x + 1, int_y + 1),
            -1.0,
            1.0,
        )
    }

    /// Genera ruido fractal Browniano (FBM).
    ///
    /// # Parámetros:
    /// - `x`: La coordenada `x`.
    /// - `y`: La coordenada `y`.
    /// - `per`: El valor de permutación.
    /// - `octs`: El número de octavas.
    ///
    /// # Retorno:
    /// El valor de FBM calculado.
    pub fn fbm(x: f32, y: f32, per: u32, octs: u32) -> f32 {
        let mut val: f32 = 0.0;

        for o in 0..octs {
            val += f32::powi(0.5, o as i32)
                * perlin_noise(
                    x * f32::powi(2.0, o as i32),
                    y * f32::powi(2.0, o as i32),
                    (per as f32 * f32::powi(2.0, o as i32)) as u32,
                );
        }
        val
    }

    /// Crea datos de ruido para un mundo virtual.
    ///
    /// # Parámetros:
    /// - `width`: El ancho del mundo.
    /// - `height`: La altura del mundo.
    /// - `frequency`: La frecuencia del ruido.
    ///
    /// # Retorno:
    /// Un vector con los valores de ruido generados.
    pub fn create_world_noise_data(width: u32, height: u32, frequency: f32) -> Vec<f32> {
        let mut data: Vec<f32> = Vec::with_capacity((width * height) as usize);

        for y in 0..height {
            for x in 0..width {
                data.push(fbm(
                    (x as f32) * frequency,
                    (y as f32) * frequency,
                    (width as f32 * frequency) as u32,
                    4,
                ));
            }
        }
        data
    }
}

/// Módulo que contiene la implementación de un `ThreadPool` simple.
pub(crate) mod threadpool {
    use std::{
        sync::{mpsc, Arc, Mutex},
        thread,
    };

    /// Estructura `Worker` que representa un hilo trabajador en el pool de hilos.
    pub struct Worker {
        id: usize,
        thread: thread::JoinHandle<()>,
    }

    impl Worker {
        /// Crea un nuevo trabajador.
        ///
        /// # Parámetros:
        /// - `id`: El identificador del trabajador.
        /// - `receiver`: Un receptor que recibe trabajos para ejecutar.
        ///
        /// # Retorno:
        /// Un nuevo `Worker`.
        pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
            let thread = thread::spawn(move || loop {
                let receiver = receiver.lock().unwrap();
                if let Ok(job) = receiver.recv() {
                    job();
                }
            });
            Worker { id, thread }
        }
    }

    /// Estructura `ThreadPool` que administra múltiples trabajadores.
    pub struct ThreadPool {
        workers: Vec<Worker>,
        sender: mpsc::Sender<Job>,
    }

    /// Tipo de alias para un trabajo encolado en el `ThreadPool`.
    type Job = Box<dyn FnOnce() + Send + 'static>;

    impl ThreadPool {
        /// Ejecuta un trabajo en el `ThreadPool`.
        ///
        /// # Parámetros:
        /// - `f`: El trabajo a ejecutar.
        pub fn execute<F>(&self, f: F)
        where
            F: FnOnce() + Send + 'static,
        {
            let job = Box::new(f);
            self.sender.send(job).unwrap();
        }

        /// Crea un nuevo `ThreadPool`.
        ///
        /// # Parámetros:
        /// - `size`: El número de trabajadores en el pool.
        ///
        /// # Retorno:
        /// Un nuevo `ThreadPool`.
        pub fn new(size: usize) -> ThreadPool {
            assert!(size > 0);

            let (sender, receiver) = mpsc::channel();
            let receiver = Arc::new(Mutex::new(receiver));

            let mut workers = Vec::with_capacity(size);

            for id in 0..size {
                workers.push(Worker::new(id, Arc::clone(&receiver)))
            }
            ThreadPool { workers, sender }
        }
    }
}

/// Rasgo para obtener el chunk basado en la posición.
pub trait ChunkFromPosition {
    fn get_chunk_from_position_absolute(&self) -> (i32, i32);
}

/// Rasgo para obtener la posición relativa a partir de una posición absoluta.
pub trait RelativeFromAbsolute {
    fn relative_from_absolute(&self) -> glam::Vec3;
}

impl RelativeFromAbsolute for glam::Vec3 {
    /// Convierte una posición absoluta a una posición relativa dentro de un chunk.
    ///
    /// # Retorno:
    /// La posición relativa dentro del chunk.
    fn relative_from_absolute(&self) -> Vec3 {
        vec3(
            ((f32::floor(self.x) % CHUNK_SIZE as f32) + CHUNK_SIZE as f32) % CHUNK_SIZE as f32,
            f32::max(f32::floor(self.y), 0.0),
            ((f32::floor(self.z) % CHUNK_SIZE as f32) + CHUNK_SIZE as f32) % CHUNK_SIZE as f32,
        )
    }
}

impl ChunkFromPosition for glam::Vec3 {
    /// Obtiene el chunk basado en la posición absoluta.
    ///
    /// # Retorno:
    /// Una tupla con las coordenadas `(x, z)` del chunk.
    fn get_chunk_from_position_absolute(&self) -> (i32, i32) {
        (
            (f32::floor(self.x / CHUNK_SIZE as f32)) as i32,
            (f32::floor(self.z / CHUNK_SIZE as f32)) as i32,
        )
    }
}

/// Módulo de pruebas.
mod tests {
    use crate::utils::{ChunkFromPosition, RelativeFromAbsolute};

    #[test]
    fn should_get_the_correct_chunk_from_position_absolute() {
        let absolute_position = glam::vec3(17.0, 0.0, 20.0);
        assert_eq!(absolute_position.get_chunk_from_position_absolute(), (1, 1));
        let absolute_position = glam::vec3(32.0, 0.0, 20.0);
        assert_eq!(absolute_position.get_chunk_from_position_absolute(), (2, 1));
        let absolute_position = glam::vec3(-5.0, 0.0, -20.0);
        assert_eq!(
            absolute_position.get_chunk_from_position_absolute(),
            (-1, -2)
        );
    }

    #[test]
    fn should_get_the_correct_relative_position() {
        let absolute_position = glam::vec3(17.0, 0.0, 20.0);
        assert_eq!(
            absolute_position.relative_from_absolute(),
            glam::vec3(1.0, 0.0, 4.0)
        );
        let absolute_position = glam::vec3(-1.0, 0.0, -1.0);
        assert_eq!(
            absolute_position.relative_from_absolute(),
            glam::vec3(15.0, 0.0, 15.0)
        );
    }
}
