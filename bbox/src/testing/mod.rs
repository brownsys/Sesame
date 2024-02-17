// TODO(babman): turn this into a proper testing util suite
// 1. TestPolicy<P: Policy>  differs to P but marks that this is testing
// 2. impl BBox<T, TestPolicy<P>> allow unboxing etc
// 3. Create a BBoxClient that wraps around rocket Client