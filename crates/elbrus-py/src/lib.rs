use pyo3::prelude::*;

#[pymodule]
fn elbrus(_m: &Bound<'_, PyModule>) -> PyResult<()> {
    Ok(())
}
