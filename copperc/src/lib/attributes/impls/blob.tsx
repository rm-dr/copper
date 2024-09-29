import { attrTypeInfo } from "..";
import { Binary } from "lucide-react";
import { BasicForm } from "../_basicform";

export const _blobAttrType: attrTypeInfo = {
	pretty_name: "Blob",
	serialize_as: "Blob",
	icon: <Binary />,
	create_params: {
		form: (params) => BasicForm({ attr_type: { type: "Blob" }, ...params }),
	},
};
