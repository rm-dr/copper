import { ReactNode } from "react";
import style from "./titlebar.module.scss";

export default function TitleBar(params: { text: string; right?: ReactNode }) {
	return (
		<>
			<div className={style.title_bar}>
				<div className={style.title_bar_text}>{params.text}</div>
				{params.right === undefined ? null : (
					<div className={style.title_bar_right}>{params.right}</div>
				)}
			</div>
		</>
	);
}
