import {
	XIconAttrBinary,
	XIconAttrBlob,
	XIconAttrFloat,
	XIconAttrInt,
	XIconAttrPosInt,
} from "@/app/components/icons";
import { Text } from "@mantine/core";
import { ReactElement, ReactNode } from "react";
import { _textAttrType } from "./text";
import { _hashAttrType } from "./hash";
import { _refAttrType } from "./reference";

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
	value_preview?: (params: { attr: any }) => ReactElement;

	// How to display the old value of attr
	// in the editor. `attr` param is the object
	// returned by the api.
	old_value?: (params: { attr: any }) => ReactElement;

	// Inline value editor
	new_value?: (params: {
		attr: any;
		onChange: (value: any) => void;
	}) => ReactElement;

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

	{
		pretty_name: "Binary",
		serialize_as: "Binary",
		icon: <XIconAttrBinary />,
		extra_params: null,
	},

	{
		pretty_name: "Blob",
		serialize_as: "Blob",
		icon: <XIconAttrBlob />,
		extra_params: null,

		value_preview: (params) => (
			<Text c="dimmed" fs="italic">{`Blob id=${params.attr.handle}`}</Text>
		),
	},

	{
		pretty_name: "Integer",
		serialize_as: "Integer",
		icon: <XIconAttrInt />,
		extra_params: null,

		value_preview: (params) => params.attr.value,
		old_value: (params) => params.attr.value,
	},

	{
		pretty_name: "Positive Integer",
		serialize_as: "PositiveInteger",
		icon: <XIconAttrPosInt />,
		extra_params: null,

		value_preview: (params) => params.attr.value,
		old_value: (params) => params.attr.value,
	},

	{
		pretty_name: "Float",
		serialize_as: "Float",
		icon: <XIconAttrFloat />,
		extra_params: null,

		value_preview: (params) => params.attr.value,
		old_value: (params) => params.attr.value,
	},
];
