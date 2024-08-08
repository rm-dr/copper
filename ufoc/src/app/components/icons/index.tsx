import {
	IconBinaryTree,
	IconCheck,
	IconCpu,
	IconFile,
	IconHexagonMinus,
	IconMenu2,
	IconSquareAsterisk,
	IconTrash,
	IconUpload,
	IconX,
} from "@tabler/icons-react";
import { CSSProperties } from "react";

const defaultStyles: CSSProperties | undefined = {
	strokeWidth: "1.5px",
	width: "100%",
	height: "1.5rem",
	margin: "auto",
};

// This module lets us apply default styling to tabler icons,
// and makes it easy to replace our icon provider.

export const XIconHexMinus = ({ style }: { style?: CSSProperties }) => {
	return <IconHexagonMinus style={{ ...defaultStyles, ...style }} />;
};

export const XIconBinaryTree = ({ style }: { style?: CSSProperties }) => {
	return <IconBinaryTree style={{ ...defaultStyles, ...style }} />;
};

export const XIconCheck = ({ style }: { style?: CSSProperties }) => {
	return <IconCheck style={{ ...defaultStyles, ...style }} />;
};

export const XIconFile = ({ style }: { style?: CSSProperties }) => {
	return <IconFile style={{ ...defaultStyles, ...style }} />;
};

export const XIconUpload = ({ style }: { style?: CSSProperties }) => {
	return <IconUpload style={{ ...defaultStyles, ...style }} />;
};

export const XIconMenu = ({ style }: { style?: CSSProperties }) => {
	return <IconMenu2 style={{ ...defaultStyles, ...style }} />;
};

export const XIconCpu = ({ style }: { style?: CSSProperties }) => {
	return <IconCpu style={{ ...defaultStyles, ...style }} />;
};

export const XIconAst = ({ style }: { style?: CSSProperties }) => {
	return <IconSquareAsterisk style={{ ...defaultStyles, ...style }} />;
};

export const XIconTrash = ({ style }: { style?: CSSProperties }) => {
	return <IconTrash style={{ ...defaultStyles, ...style }} />;
};

export const XIconX = ({ style }: { style?: CSSProperties }) => {
	return <IconX style={{ ...defaultStyles, ...style }} />;
};
