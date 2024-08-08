import { attrTypeInfo } from ".";
import { Text } from "@mantine/core";
import { ppBytes } from "../ppbytes";
import {
	_PanelBodyAudio,
	_PanelBodyImage,
	_PanelBodyUnknown,
	_PanelBottom,
} from "./blob";
import { XIcon } from "@/app/components/icons";
import { IconBinary } from "@tabler/icons-react";

export const _binaryAttrType: attrTypeInfo = {
	pretty_name: "Binary",
	serialize_as: "Binary",
	icon: <XIcon icon={IconBinary} />,
	extra_params: null,

	value_preview: (params) => {
		if (params.attr_value.type !== "Binary") {
			return <>Unreachable!</>;
		}

		if (params.attr_value.size == null) {
			return (
				<Text c="dimmed" fs="italic">
					no value
				</Text>
			);
		} else {
			return (
				<Text c="dimmed" fs="italic">{`Binary (${ppBytes(
					params.attr_value.size,
				)})`}</Text>
			);
		}
	},

	editor: {
		type: "panel",

		panel_body: (params) => {
			const data_url =
				"/api/item/attr?" +
				new URLSearchParams({
					dataset: params.dataset,
					class: params.class.toString(),
					item_idx: params.item_idx.toString(),
					attr: params.attr_value.attr.handle.toString(),
				});

			if (params.attr_value.type !== "Binary") {
				return <>Unreachable!</>;
			}

			let inner = (
				<_PanelBodyUnknown
					src={data_url}
					icon={<XIcon icon={IconBinary} style={{ height: "5rem" }} />}
					attr_value={params.attr_value}
				/>
			);

			if (
				params.attr_value.mime != null &&
				params.attr_value.mime.startsWith("image/")
			) {
				inner = (
					<_PanelBodyImage src={data_url} attr_value={params.attr_value} />
				);
			} else if (
				params.attr_value.mime != null &&
				params.attr_value.mime.startsWith("audio/")
			) {
				inner = (
					<_PanelBodyAudio src={data_url} attr_value={params.attr_value} />
				);
			}

			return (
				<div
					style={{
						height: "100%",
						width: "100%",
						display: "flex",
						flexDirection: "column",
					}}
				>
					<div
						style={{
							width: "100%",
							flexGrow: 1,
							padding: params.inner !== true ? "0.5rem" : undefined,
							cursor: "zoom-in",
						}}
					>
						<a
							target="_blank"
							href={data_url}
							rel="noopener noreferrer"
							style={{ width: "100%", height: "100%", cursor: "inherit" }}
						>
							{inner}
						</a>
					</div>
					{params.inner !== true ? (
						<_PanelBottom attr_value={params.attr_value} />
					) : null}
				</div>
			);
		},
	},
};
