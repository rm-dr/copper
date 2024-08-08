import { ReactElement, ReactNode } from "react";
import { _textAttrType } from "./text";
import { _hashAttrType } from "./hash";
import { _refAttrType } from "./reference";
import { _binaryAttrType } from "./binary";
import { _blobAttrType } from "./blob";
import { _posintAttrType } from "./positiveinteger";
import { _floatAttrType } from "./float";
import { _intAttrType } from "./integer";
import { components } from "../api/openapi";

/*
	Definitions of all attribute types we support
*/

export type attrTypeInfo = {
	// Pretty name to display to user
	pretty_name: string;

	// The name of this data type in ufo's api
	serialize_as: string;

	// Icon to use for attrs of this type
	icon: ReactNode;

	// How to display the value of this attr
	// in the item table. This should be compact
	// and non-interactive.
	value_preview: (params: {
		attr_value: components["schemas"]["ItemListData"];
	}) => ReactElement;

	editor:
		| {
				type: "inline";
				// How to display the old value of attr
				// in the editor. `attr` param is the object
				// returned by the api.
				old_value: (params: {
					// Key for the react node this returns.
					// This is NECESSARY, data won't always update
					// when we select a new item if this is omitted.
					key: string;
					attr_value: components["schemas"]["ItemListData"];
				}) => ReactElement;

				// Inline value editor
				new_value: (params: {
					// Key for the react node this returns.
					// This is NECESSARY, data won't always update
					// when we select a new item if this is omitted.
					key: string;
					attr_value: components["schemas"]["ItemListData"];
					onChange: (value: any) => void;
				}) => ReactElement;
		  }
		| {
				type: "panel";

				panel_body: (params: {
					dataset: string;
					class: number;
					item_idx: number;
					attr_value: components["schemas"]["ItemListData"];
				}) => ReactElement;

				panel_bottom: (params: {
					dataset: string;
					class: number;
					item_idx: number;
					attr_value: components["schemas"]["ItemListData"];
				}) => ReactElement;
		  };

	// TODO: fix these types (no any)

	// Extra parameter elements for this type.
	// Consumes a function is called whenever parameters change,
	// returns html that controls this input.
	extra_params: null | {
		/// Check input state. This is called when the "submit"
		/// button is pressed. If `setErrorMessage` is not `null`
		/// the error is displayed.
		///
		/// If this returns `true`, the request proceeds.
		/// If it is `false`, fail and show the error.
		inputs_ok: (params: {
			state: Object;
			setErrorMessage: (message: null | Object) => void;
		}) => boolean;

		/// A react component that contains extra input for this attr
		node: (params: {
			/// Called when any input is changed
			onChange: (state: null | any) => void;

			dataset_name: string;

			/// This is usually a string, but can be a table for attrs
			/// that need multiple extra inputs.
			setErrorMessage: (message: null | any) => void;
			errorMessage: null | any;
		}) => ReactElement;
	};
};

export const attrTypes: attrTypeInfo[] = [
	_textAttrType,
	_hashAttrType,
	_refAttrType,
	_binaryAttrType,
	_blobAttrType,
	_posintAttrType,
	_floatAttrType,
	_intAttrType,
];
