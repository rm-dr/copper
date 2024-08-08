import { IconServer2 } from "@tabler/icons-react";
import { ReactNode } from "react";
import { XIcon } from "../components/icons";

/*
	Definitions of all dataset types we support
*/

export const datasetTypes: {
	// Pretty name to display to user
	pretty_name: string;

	// The name of this type in copper's api
	serialize_as: string;

	// Icon to use for datasets of this type
	icon: ReactNode;

	// Extra parameter elements for this dataset
	// (Currently unused. We'll need this later.)
	extra_params: any;
}[] = [
	{
		pretty_name: "Local",
		serialize_as: "Local",
		icon: <XIcon icon={IconServer2} />,
		extra_params: null,
	},
];
