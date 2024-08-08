import { XIconAttrReference } from "@/app/components/icons";
import { attrTypeInfo } from ".";
import { Text } from "@mantine/core";
import { ClassSelector } from "@/app/components/apiselect/class";

export const _refAttrType: attrTypeInfo = {
	pretty_name: "Reference",
	serialize_as: "Reference",
	icon: <XIconAttrReference />,
	extra_params: {
		inputs_ok: checkRef,
		node: RefParams,
	},

	value_preview: (params) => {
		if (params.attr.item === null) {
			return (
				<Text c="dimmed" fs="italic">
					no value
				</Text>
			);
		} else {
			return (
				<Text c="dimmed">
					Reference to{" "}
					<Text c="dimmed" fs="italic" span>
						{params.attr.class}
					</Text>
				</Text>
			);
		}
	},

	editor: {
		type: "panel",

		panel_body: (params) => {
			return (
				<>
					<div
						style={{
							display: "flex",
							flexDirection: "column",
							justifyContent: "center",
							alignItems: "center",
							color: "var(--mantine-color-dimmed)",
							height: "100%",
						}}
					>
						<div>
							<XIconAttrReference style={{ height: "5rem" }} />
						</div>
						<div>
							Reference to{" "}
							<Text c="dimmed" fs="italic" span>
								{params.attr_val.class}
							</Text>
						</div>
					</div>
				</>
			);
		},
	},
};

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
