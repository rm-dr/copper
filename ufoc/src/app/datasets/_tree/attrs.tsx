import {
	XIconAttrBinary,
	XIconAttrBlob,
	XIconAttrFloat,
	XIconAttrHash,
	XIconAttrInt,
	XIconAttrPosInt,
	XIconAttrReference,
	XIconAttrText,
} from "@/app/components/icons";
import { Select } from "@mantine/core";
import { ReactElement, ReactNode, useState } from "react";

// Server-compatible attr definitions

export const attrTypes: {
	// Pretty name to display to user
	pretty_name: string;

	// The name of this data type in ufo's api
	serialize_as: string;

	// Icon to use for attrs of this type
	icon: ReactNode;

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
			onChange: (state: null | any) => void;

			/// This is usually a string, but can be a table for attrs
			/// that need multiple extra inputs.
			setErrorMessage: (message: null | any) => void;
			errorMessage: null | any;
		}) => ReactElement;
	};
}[] = [
	{
		pretty_name: "Text",
		serialize_as: "Text",
		icon: <XIconAttrText />,
		extra_params: null,
	},

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
	},

	{
		pretty_name: "Integer",
		serialize_as: "Integer",
		icon: <XIconAttrInt />,
		extra_params: null,
	},

	{
		pretty_name: "Positive Integer",
		serialize_as: "PositiveInteger",
		icon: <XIconAttrPosInt />,
		extra_params: null,
	},

	{
		pretty_name: "Float",
		serialize_as: "Float",
		icon: <XIconAttrFloat />,
		extra_params: null,
	},

	{
		pretty_name: "Hash",
		serialize_as: "Hash",
		icon: <XIconAttrHash />,
		extra_params: {
			inputs_ok: checkHash,
			node: HashParams,
		},
	},

	// TODO: reference type
];

function checkHash(params: {
	state: any;
	setErrorMessage: (message: null | any) => void;
}): boolean {
	if (params.state === null) {
		params.setErrorMessage("Hash type is required");
		return false;
	} else if (params.state.hash_type === null) {
		params.setErrorMessage("Hash type is required");
		return false;
	}

	return true;
}

function HashParams(params: {
	onChange: (state: null | any) => void;
	setErrorMessage: (message: null | any) => void;
	errorMessage: null | any;
}) {
	return (
		<Select
			required={true}
			placeholder={"select hash type"}
			data={[
				// Hash types the server supports
				{ label: "MD5", value: "MD5", disabled: false },
				{ label: "SHA256", value: "SHA256", disabled: false },
				{ label: "SHA512", value: "SHA512", disabled: false },
			]}
			clearable
			error={params.errorMessage !== null}
			onChange={(v) => {
				params.setErrorMessage(null);
				if (v == null) {
					params.onChange({ hash_type: null });
				} else {
					params.onChange({ hash_type: v });
				}
			}}
		/>
	);
}

