import type { Dispatch, SetStateAction } from "react";

import {
  process_event,
  handle_response,
  view,
} from "shared/shared";
import type {
  Effect,
  Event,
    HttpResult,
  //   SseResponse,
} from "shared_types/types/shared_types";
import {
  EffectVariantRender,
  ViewModel,
    EffectVariantHttp,
  HttpResultVariantOk,
  //   EffectVariantServerSentEvents,
  Request,
} from "shared_types/types/shared_types";
import {
  BincodeSerializer,
  BincodeDeserializer,
} from "shared_types/bincode/mod";

import { request as http } from "./http";
// import { request as sse } from "./sse";

type Response = HttpResult;

export function update(
  event: Event,
  callback: Dispatch<SetStateAction<ViewModel>>
) {
  console.log("event", event);

  const serializer = new BincodeSerializer();
  event.serialize(serializer);

  const effects = process_event(serializer.getBytes());

  const requests = deserializeRequests(effects);
  for (const { id, effect } of requests) {
    processEffect(id, effect, callback);
  }
}

async function processEffect(
  id: number,
  effect: Effect,
  callback: Dispatch<SetStateAction<ViewModel>>
) {
  console.log("effect", effect);

  switch (effect.constructor) {
    case EffectVariantRender: {
      callback(deserializeView(view()));
      break;
    }
    case EffectVariantHttp: {
      const request = (effect as EffectVariantHttp).value;
      const result = await http(request);
      if (result instanceof HttpResultVariantOk) {
        respond(id, result, callback);
      }
      break;
    }
    // case EffectVariantServerSentEvents: {
    //   const request = (effect as EffectVariantServerSentEvents).value;
    //   for await (const response of sse(request)) {
    //     respond(id, response, callback);
    //   }
    //   break;
    // }
  }
}

function respond(
  id: number,
  response: Response,
  callback: Dispatch<SetStateAction<ViewModel>>,
) {
  const serializer = new BincodeSerializer();
  response.serialize(serializer);

  const effects = handle_response(id, serializer.getBytes());

  const requests = deserializeRequests(effects);
  for (const { id, effect } of requests) {
    processEffect(id, effect, callback);
  }
}

export function deserializeRequests(bytes: Uint8Array): Request[] {
  const deserializer = new BincodeDeserializer(bytes);
  const len = deserializer.deserializeLen();
  const requests: Request[] = [];
  for (let i = 0; i < len; i++) {
    const request = Request.deserialize(deserializer);
    requests.push(request);
  }
  return requests;
}

export function deserializeView(bytes: Uint8Array): ViewModel {
  return ViewModel.deserialize(new BincodeDeserializer(bytes));
}
