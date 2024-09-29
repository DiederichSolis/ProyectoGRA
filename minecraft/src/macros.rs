/// Macro `perf` para medir el tiempo de ejecución de un bloque de código.
/// 
/// # Parámetros:
/// - `$start`: Expresión que representa el momento en que se inició la ejecución (debe ser un `Instant`).
/// - `$fn_name`: Nombre de la función o bloque que se está midiendo, representado como una cadena (`&str`).
/// 
/// # Descripción:
/// Esta macro calcula el tiempo que ha pasado desde el momento indicado por `$start` hasta el presente,
/// utilizando `Instant::now()`. Luego, imprime el nombre de la función (`$fn_name`) y el tiempo transcurrido 
/// en segundos con precisión de milisegundos (`as_secs_f64()`).
/// 
/// # Ejemplo:
/// ```rust
/// let start = Instant::now();
/// // Aquí va el código que quieres medir.
/// perf!(start, "nombre_de_función");
/// ```
/// 
/// El resultado impreso será algo como:
/// `PERF: nombre_de_función - 0.00234`
/// 
/// Esto indica que la ejecución del bloque duró aproximadamente 0.00234 segundos.
#[macro_export]
macro_rules! perf {
    ($start:expr, $fn_name:expr) => {
        let end = Instant::now();
        println!("PERF: {} - {}", $fn_name, (end - $start).as_secs_f64())
    };
}
