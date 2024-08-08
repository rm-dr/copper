import { CSSProperties } from "react";

const defaultStyles: CSSProperties | undefined = {
	strokeWidth: "1.5px",
	width: "100%",
	height: "1.5rem",
	margin: "auto",
};

// This module lets us apply default styling to tabler icons,
// and makes it easy to replace our icon provider.

export const XIcon = (params: { icon: any; style?: CSSProperties }) => {
	return <params.icon style={{ ...defaultStyles, ...params.style }} />;
};
