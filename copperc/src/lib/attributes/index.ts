import { ReactElement, ReactNode } from "react";

import { _textAttrType } from "./impls/text";
import { _booleanAttrType } from "./impls/boolean";
import { _blobAttrType } from "./impls/blob";
import { _floatAttrType } from "./impls/float";
import { _integerAttrType } from "./impls/integer";
import { _hashAttrType } from "./impls/hash";
import { _referenceAttrType } from "./impls/reference";
import { components } from "../api/openapi";

export type attrTypeInfo<
	SerializeAs extends
		components["schemas"]["ItemAttrData"]["type"] = components["schemas"]["ItemAttrData"]["type"],
> = {
	// Pretty name to display to user
	pretty_name: string;

	// The name of this data type in copper's api
	serialize_as: SerializeAs;

	// Icon to use for attrs of this type
	icon: ReactNode;

	// Extra parameter elements for this type.
	// Consumes a function is called whenever parameters change,
	// returns html that controls this input.
	create_params: {
		/// The form we use to create this attr.
		/// This should contain everything (including buttons),
		/// except for the attribute type selector.
		form: (params: {
			dataset_id: number;
			class_id: number;
			onSuccess: () => void;
			close: () => void;
		}) => ReactElement;
	};

	table_cell: (
		value: components["schemas"]["ItemAttrData"],
	) => null | ReactNode;
};

export const attrTypes = [
	_textAttrType,
	_booleanAttrType,
	_blobAttrType,
	_floatAttrType,
	_integerAttrType,
	_hashAttrType,
	_referenceAttrType,
] as const;

export const dataTypes = attrTypes.map((x) => x.serialize_as);
