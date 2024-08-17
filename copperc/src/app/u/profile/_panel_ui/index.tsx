import styles from "../page.module.scss";
import { Panel, PanelTitle } from "@/app/components/panel";
import { IconColorPicker, IconPaint } from "@tabler/icons-react";
import { XIcon } from "@/app/components/icons";
import { Button, ColorPicker, Text } from "@mantine/core";
import { useEffect, useState } from "react";
import { APIclient } from "@/app/_util/api";
import { useUserInfoStore } from "@/app/_util/userinfo";

export function useUiPanel(params: {}) {
	return {
		node: (
			<Panel
				panel_id={styles.panel_id_ui}
				icon={<XIcon icon={IconPaint} />}
				title={"Interface"}
			>
				<UiPanelColor />
			</Panel>
		),
	};
}

function UiPanelColor(params: {}) {
	const user = useUserInfoStore((state) => state.user_info);
	const [isLoading, setLoading] = useState(false);
	const [errorMessage, setErrorMessage] = useState<string | null>(null);
	const [color, setColor] = useState(user?.color || "#aaaaaa");
	const set_color = useUserInfoStore((state) => state.set_color);
	const preview_color = useUserInfoStore((state) => state.preview_color);

	useEffect(() => {
		setColor(user?.color || "#aaaaaa");
		set_color(user?.color || "#aaaaaa");
	}, [set_color, setColor, user?.color]);

	return (
		<>
			<PanelTitle
				icon={<XIcon icon={IconColorPicker} />}
				title={"Primary color"}
			/>
			<div className={styles.settings_container}>
				Pick a primary color for your UI.
				<ColorPicker
					fullWidth
					format="hex"
					swatches={[
						"#2e2e2e",
						"#868e96",
						"#fa5252",
						"#e64980",
						"#be4bdb",
						"#7950f2",
						"#4c6ef5",
						"#228be6",
						"#15aabf",
						"#12b886",
						"#40c057",
						"#82c91e",
						"#fab005",
						"#fd7e14",
					]}
					value={color}
					onChange={(x) => {
						setColor(x);
						preview_color(x);
					}}
				/>
				<Button.Group>
					<Button
						variant="light"
						fullWidth
						color="red"
						onClick={() => {
							setColor(user?.color || "#aaaaaa");
							set_color(user?.color || "#aaaaaa");
							setErrorMessage(null);
						}}
						disabled={isLoading}
					>
						Reset
					</Button>
					<Button
						variant="filled"
						color="green"
						fullWidth
						onClick={() => {
							setLoading(true);
							setErrorMessage(null);

							// Minimum delay so user sees loader
							Promise.all([
								new Promise((r) => setTimeout(r, 500)),
								APIclient.POST("/auth/user/set_info", {
									body: {
										user: 1 as any,
										color: { action: "Set", color: color },
										email: { action: "Unchanged" },
									},
								}),
							])
								.then(([_, { data, error }]) => {
									if (error !== undefined) {
										throw error;
									}

									setLoading(false);
									set_color(color);
								})
								.catch((err) => {
									setLoading(false);
									setErrorMessage(err);
								});
						}}
						loading={isLoading}
					>
						Save
					</Button>
				</Button.Group>
				<Text c="red" ta="center">
					{errorMessage}
				</Text>
			</div>
		</>
	);
}
