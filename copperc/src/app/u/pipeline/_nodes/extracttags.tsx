import { Node, NodeProps } from "@xyflow/react";
import { BaseNode } from "./base";
import { NodeDef } from ".";

type ExtractTagsNodeType = Node<Record<string, never>, "extracttags">;

function ExtractTagsNodeElement({ id }: NodeProps<ExtractTagsNodeType>) {
	return (
		<>
			<BaseNode
				id={id}
				title={"Extract tags"}
				inputs={[{ id: "data", type: "Blob", tooltip: "Audio data" }]}
				outputs={[
					{ id: "Album", type: "Text", tooltip: "Album" },
					{ id: "AlbumArtist", type: "Text", tooltip: "Album artist" },
					{ id: "Comment", type: "Text", tooltip: "Comment" },
					{ id: "ReleaseDate", type: "Text", tooltip: "Release date" },
					{ id: "DiskNumber", type: "Text", tooltip: "Disk number" },
					{ id: "DiskTotal", type: "Text", tooltip: "Total disks" },
					{ id: "Genre", type: "Text", tooltip: "Genre" },
					{ id: "ISRC", type: "Text", tooltip: "ISRC" },
					{ id: "Lyrics", type: "Text", tooltip: "Lyrics" },
					{ id: "TrackNumber", type: "Text", tooltip: "Track number" },
					{ id: "TrackTotal", type: "Text", tooltip: "Total tracks" },
					{ id: "Title", type: "Text", tooltip: "Track title" },
					{ id: "Artist", type: "Text", tooltip: "Artist" },
					{ id: "Year", type: "Text", tooltip: "Year" },
				]}
			></BaseNode>
		</>
	);
}

export const ExtractTagsNode: NodeDef<ExtractTagsNodeType> = {
	xyflow_node_type: "extracttags",
	copper_node_type: "ExtractTags",
	node: ExtractTagsNodeElement,

	getInputs: () => [{ id: "data", type: "Blob" }],
	getOutputs: () => [
		{ id: "Album", type: "Text" },
		{ id: "AlbumArtist", type: "Text" },
		{ id: "Comment", type: "Text" },
		{ id: "ReleaseDate", type: "Text" },
		{ id: "DiskNumber", type: "Text" },
		{ id: "DiskTotal", type: "Text" },
		{ id: "Genre", type: "Text" },
		{ id: "ISRC", type: "Text" },
		{ id: "Lyrics", type: "Text" },
		{ id: "TrackNumber", type: "Text" },
		{ id: "TrackTotal", type: "Text" },
		{ id: "Title", type: "Text" },
		{ id: "Artist", type: "Text" },
		{ id: "Year", type: "Text" },
	],

	initialData: {},

	serialize: () => ({
		tags: {
			parameter_type: "List",
			value: [
				{ parameter_type: "String", value: "AlbumArtist" },
				{ parameter_type: "String", value: "Comment" },
				{ parameter_type: "String", value: "ReleaseDate" },
				{ parameter_type: "String", value: "DiskNumber" },
				{ parameter_type: "String", value: "DiskTotal" },
				{ parameter_type: "String", value: "Genre" },
				{ parameter_type: "String", value: "ISRC" },
				{ parameter_type: "String", value: "Lyrics" },
				{ parameter_type: "String", value: "TrackNumber" },
				{ parameter_type: "String", value: "TrackTotal" },
				{ parameter_type: "String", value: "Title" },
				{ parameter_type: "String", value: "Artist" },
				{ parameter_type: "String", value: "Year" },
			],
		},
	}),

	deserialize: async () => ({}),
};
