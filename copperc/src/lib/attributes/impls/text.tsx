import { attrTypeInfo } from "..";
import { LetterText } from "lucide-react";
import { BasicForm } from "../_basicform";

export const _textAttrType: attrTypeInfo<"Text"> = {
	pretty_name: "Text",
	serialize_as: "Text",
	icon: <LetterText />,
	create_params: {
		form: (params) => BasicForm({ attr_type: { type: "Text" }, ...params }),
	},

	table_cell: (value) => {
		if (value.type !== "Text") {
			return null;
		}

		return value.value;
	},
};
