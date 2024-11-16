// Hack types to enforce `attrTypes`
type NonEmptyArray<T> = [T, ...T[]];
type MustInclude<T, U extends T[]> = [T] extends [U[keyof U]] ? U : never;

/**
 * Hack that allows us to create a type-checked array
 * that contains every variant of a string union.
 */
// TODO: uniqueness
export function stringUnionToArray<T>() {
	return <U extends NonEmptyArray<T>>(...elements: MustInclude<T, U>) =>
		elements;
}
