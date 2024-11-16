import { ReactElement, ReactNode } from "react";

import { _textAttrType } from "./impls/text";
import { _booleanAttrType } from "./impls/boolean";
import { _blobAttrType } from "./impls/blob";
import { _floatAttrType } from "./impls/float";
import { _integerAttrType } from "./impls/integer";
import { _hashAttrType } from "./impls/hash";
import { _referenceAttrType } from "./impls/reference";
import { components } from "../api/openapi";
import { stringUnionToArray } from "../util";

export type AttrDataType =
	components["schemas"]["AttributeInfo"]["data_type"]["type"];

export const attrDataTypes = stringUnionToArray<AttrDataType>()(
	"Text",
	"Integer",
	"Float",
	"Boolean",
	"Hash",
	"Blob",
	"Reference",
);

export type attrTypeInfo<SerializeAs extends AttrDataType = AttrDataType> = {
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
		// The form we use to create this attr.
		// This should contain everything (including buttons),
		// except for the attribute type selector.
		form: (params: {
			dataset_id: number;
			class_id: number;
			onSuccess: () => void;
			close: () => void;
		}) => ReactElement;
	};

	// TODO: what does null mean?
	// TODO: stricter types
	table_cell: (params: {
		value: Extract<
			components["schemas"]["ItemAttrData"],
			{ type: SerializeAs }
		>;

		dataset: components["schemas"]["DatasetInfo"];
	}) => null | ReactNode;

	editor:
		| {
				type: "inline";

				old_value: (
					value: Extract<
						components["schemas"]["ItemAttrData"],
						{ type: SerializeAs }
					>,
				) => null | ReactNode;

				new_value: (params: {
					value: Extract<
						components["schemas"]["ItemAttrData"],
						{ type: SerializeAs }
					> | null;

					onChange: (
						value:
							| Extract<
									components["schemas"]["ItemAttrData"],
									{
										type: SerializeAs;
									}
							  >
							| {
									type: SerializeAs;
									value: null;
							  },
					) => void;
				}) => null | ReactNode;
		  }
		| {
				type: "panel";

				panel_body: (params: {
					item_id: number;
					attr_id: number;

					value: Extract<
						components["schemas"]["ItemAttrData"],
						{ type: SerializeAs }
					>;

					// If this is true, this is drawn inside another panel.
					// Exclude extra padding and toolbars.
					inner?: boolean;
				}) => ReactNode;
		  };
};

export function getAttrTypeInfo<T extends AttrDataType = AttrDataType>(
	type: T,
): attrTypeInfo<T> {
	const x = {
		[_textAttrType.serialize_as]: _textAttrType,
		[_booleanAttrType.serialize_as]: _booleanAttrType,
		[_blobAttrType.serialize_as]: _blobAttrType,
		[_floatAttrType.serialize_as]: _floatAttrType,
		[_integerAttrType.serialize_as]: _integerAttrType,
		[_hashAttrType.serialize_as]: _hashAttrType,
		[_referenceAttrType.serialize_as]: _referenceAttrType,
	}[type as string];

	if (x === undefined) {
		const msg = `attr type ${type} isn't fully defined`;
		console.error(msg);
		throw new Error(msg);
	}

	return x as unknown as attrTypeInfo<T>;
}
