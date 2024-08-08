import { ReactElement, ReactNode } from "react";
import { _textAttrType } from "./text";
import { _hashAttrType } from "./hash";
import { _refAttrType } from "./reference";
import { _binaryAttrType } from "./binary";
import { _blobAttrType } from "./blob";
import { _floatAttrType } from "./float";
import { _intAttrType } from "./integer";
import { components } from "../api/openapi";

/*
	Definitions of all attribute types we support
*/

export type attrTypeInfo = {
	// Pretty name to display to user
	pretty_name: string;

	// The name of this data type in copper's api
	serialize_as: string;

	// Icon to use for attrs of this type
	icon: ReactNode;

	// How to display the value of this attr
	// in the item table. This should be compact
	// and non-interactive.
	value_preview: (params: {
		dataset: string;
		item_idx: number;
		attr_value: components["schemas"]["ItemListData"];
	}) => ReactElement;

	editor:
		| {
				type: "inline";
				// How to display the old value of attr
				// in the editor. `attr` param is the object
				// returned by the api.
				old_value: (params: {
					dataset: string;
					item_idx: number;
					attr_value: components["schemas"]["ItemListData"];
				}) => ReactElement;

				// Inline value editor
				new_value: (params: {
					dataset: string;
					item_idx: number;
					attr_value: components["schemas"]["ItemListData"];
					onChange: (value: any) => void;
				}) => ReactElement;
		  }
		| {
				type: "panel";

				panel_body: (params: {
					dataset: string;
					item_idx: number;
					attr_value: components["schemas"]["ItemListData"];

					// If this is true, this is drawn inside another panel.
					// Exclude extra padding and toolbars.
					inner?: boolean;
				}) => ReactElement;
		  };

	// TODO: fix these types (no any)

	// Extra parameter elements for this type.
	// Consumes a function is called whenever parameters change,
	// returns html that controls this input.
	params: {
		/// The form we use to create this attr.
		/// This should contain everything (including buttons),
		/// except for the attribute type selector.
		form: (params: {
			dataset_name: string;
			class: components["schemas"]["ClassInfo"];
			close: () => void;
		}) => ReactElement;
	};
};

export const attrTypes: attrTypeInfo[] = [
	_textAttrType,
	_hashAttrType,
	_refAttrType,
	_binaryAttrType,
	_blobAttrType,
	_floatAttrType,
	_intAttrType,
];
