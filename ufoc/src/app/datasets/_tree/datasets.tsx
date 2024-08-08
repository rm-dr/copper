import { XIconServer } from "@/app/components/icons";
import { ReactNode } from "react";

// Server-compatible dataset type definitions

export const datasetTypes: {
	// Pretty name to display to user
	pretty_name: string;

	// The name of this type in ufo's api
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
		icon: <XIconServer />,
		extra_params: null,
	},
];
