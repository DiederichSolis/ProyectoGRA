use std::any::Any;
use std::error::Error;

/// Trait `Saveable` define una interfaz para guardar un objeto de tipo `T`.
///
/// # Métodos requeridos:
/// - `save`: Este método se encarga de guardar el estado actual de la implementación.
///   - Retorna un `Result<(), Box<dyn Error>>`, lo que indica que puede fallar y devolver un error en caso de fallo.
///   - Si la operación es exitosa, retorna `Ok(())`.
///
/// # Ejemplo de implementación:
/// ```rust
/// impl Saveable<MyStruct> for MyStruct {
///     fn save(&self) -> Result<(), Box<dyn Error>> {
///         // Código para guardar la estructura.
///         Ok(())
///     }
/// }
/// ```
pub trait Saveable<T> {
    fn save(&self) -> Result<(), Box<dyn Error>>;
}

/// Trait `Loadable` define una interfaz para cargar un objeto de tipo `T`.
///
/// # Métodos requeridos:
/// - `load`: Este método toma un argumento de tipo `Box<dyn Any>` que puede contener cualquier tipo de dato, 
///   y retorna un `Result<T, Box<dyn Error>>`.
///   - `T`: Tipo del objeto a ser cargado.
///   - Retorna `Ok(T)` si la carga es exitosa o un `Err(Box<dyn Error>)` en caso de fallo.
///
/// # Ejemplo de implementación:
/// ```rust
/// impl Loadable<MyStruct> for MyStruct {
///     fn load(args: Box<dyn Any>) -> Result<MyStruct, Box<dyn Error>> {
///         // Código para cargar la estructura.
///         Ok(MyStruct {})
///     }
/// }
/// ```
pub trait Loadable<T> {
    fn load(args: Box<dyn Any>) -> Result<T, Box<dyn Error>>;
}
