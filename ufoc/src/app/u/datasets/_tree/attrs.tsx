import { ClassSelector } from "@/app/components/apiselect/class";
import { DatasetSelector } from "@/app/components/apiselect/dataset";
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
import { Code, Select, Text, Textarea } from "@mantine/core";
import { ReactElement, ReactNode } from "react";

// Server-compatible attr definitions

export const attrTypes: {
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
}[] = [
	{
		pretty_name: "Text",
		serialize_as: "Text",
		icon: <XIconAttrText />,
		extra_params: null,

		value_preview: (params) => {
			if (params.attr.value == "") {
				return (
					<Text c="dimmed" fs="italic">
						empty string
					</Text>
				);
			} else if (params.attr.value.trim() == "") {
				return <Text c="dimmed">""</Text>;
			} else if (params.attr.value == null) {
				return (
					<Text c="dimmed" fs="italic">
						no value
					</Text>
				);
			} else {
				return params.attr.value;
			}
		},

		old_value: (params) => {
			if (params.attr.value == "") {
				return (
					<Text c="dimmed" fs="italic">
						empty string
					</Text>
				);
			} else if (params.attr.value.trim() == "") {
				return <Text c="dimmed">""</Text>;
			} else if (params.attr.value == null) {
				return (
					<Text c="dimmed" fs="italic">
						no value
					</Text>
				);
			} else {
				return params.attr.value;
			}
		},

		new_value: (params) => {
			return (
				<Textarea
					radius="0px"
					placeholder="no value"
					autosize
					minRows={1}
					defaultValue={params.attr.value}
					onChange={params.onChange}
				/>
			);
		},
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

	{
		pretty_name: "Hash",
		serialize_as: "Hash",
		icon: <XIconAttrHash />,
		extra_params: {
			inputs_ok: checkHash,
			node: HashParams,
		},

		value_preview: (params) => (
			<Text>
				{`${params.attr.hash_type} hash: `}
				<Code>{params.attr.value}</Code>
			</Text>
		),
	},

	{
		pretty_name: "Reference",
		serialize_as: "Reference",
		icon: <XIconAttrReference />,
		extra_params: {
			inputs_ok: checkRef,
			node: RefParams,
		},

		value_preview: (params) => (
			<Text c="dimmed">
				Reference to{" "}
				<Text c="dimmed" fs="italic" span>
					{params.attr.class}
				</Text>
			</Text>
		),
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

function checkRef(params: {
	state: any;
	setErrorMessage: (message: null | any) => void;
}): boolean {
	if (params.state === null) {
		params.setErrorMessage("Reference target is required");
		return false;
	} else if (params.state.class === null) {
		params.setErrorMessage("Reference target is required");
		return false;
	}

	return true;
}

function RefParams(params: {
	onChange: (state: null | any) => void;
	dataset_name: string;
	setErrorMessage: (message: null | any) => void;
	errorMessage: null | any;
}) {
	return (
		<ClassSelector
			selectedDataset={params.dataset_name}
			onSelect={(v) => {
				if (v == null) {
					params.onChange({ class: null });
				} else {
					params.onChange({ class: parseInt(v) });
				}
			}}
		/>
	);
}
