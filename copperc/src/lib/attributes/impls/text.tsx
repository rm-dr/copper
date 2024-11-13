import { attrTypeInfo } from "..";
import { LetterText } from "lucide-react";
import { BasicForm } from "../_basicform";
import { Textarea } from "@mantine/core";

export const _textAttrType: attrTypeInfo<"Text"> = {
	pretty_name: "Text",
	serialize_as: "Text",
	icon: <LetterText />,
	create_params: {
		form: (params) => BasicForm({ attr_type: { type: "Text" }, ...params }),
	},

	table_cell: ({ value }) => {
		return (
			<div
				style={{
					paddingLeft: "0.5rem",
					width: "100%",
					overflow: "hidden",
					textOverflow: "ellipsis",
					whiteSpace: "nowrap",
					color: "var(--mantine-color-white)",
				}}
			>
				{value.value}
			</div>
		);
	},

	editor: {
		type: "inline",

		old_value: (value) => {
			return (
				<div
					style={{
						paddingLeft: "0.5rem",
						width: "100%",
						overflow: "scroll",
						textOverflow: "ellipsis",
						color: "var(--mantine-color-white)",
					}}
				>
					{value.value}
				</div>
			);
		},

		new_value: (params) => {
			return (
				<Textarea
					disabled
					radius="0px"
					placeholder="no value"
					autosize
					minRows={1}
					defaultValue={params.value?.value}
					onChange={(event) =>
						params.onChange({ type: "Text", value: event.currentTarget.value })
					}
				/>
			);
		},
	},
};
