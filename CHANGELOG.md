# Changelog

## Plotters 0.1.13 (?)

### Added
- New abstraction of backend style with `BackendStyle` trait which should be able to extend easier in the future
- Backend support features, now feature options can be used to control which backend should be supported

### Improvement
- Improved the overall code quality
- Documentation polish
- Stabilized APIs

### Fix
- Changed the oepn-close pattern to a `present` function which indicates the end of drawing one frame
- Fix the but that `ChartBuilder::title` and `ChartBuilder::margin` cannot be called at the same time && `build_ranged` now returning a result.

## Plotters 0.1.12 (2019-05-25)

The unstable version
