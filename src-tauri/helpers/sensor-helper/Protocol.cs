using System.Text.Json;
using LibreHardwareMonitor.Hardware;

namespace PcHealth.SensorHelper;

public sealed record HelperRequest(long? Id, string? Command);

public sealed record SensorRecord(
    string Name,
    string SensorType,
    string Identifier,
    string ParentIdentifier,
    string? Unit,
    double? Value);

public sealed record HardwareRecord(
    string Name,
    string HardwareType,
    string Identifier,
    string? ParentIdentifier,
    string? UpdateError,
    IReadOnlyList<SensorRecord> Sensors);

public sealed record HelperResponse(
    long? Id,
    bool Ok,
    int HelperPid,
    long SampleIndex,
    string Command,
    IReadOnlyList<HardwareRecord> Hardware,
    IReadOnlyList<string> Errors);

public static class ProtocolJson
{
    public static readonly JsonSerializerOptions Options = new()
    {
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
        PropertyNameCaseInsensitive = true,
    };
}

public static class SensorUnits
{
    public static string? For(SensorType sensorType) => sensorType switch
    {
        SensorType.Temperature => "C",
        SensorType.Current => "A",
        SensorType.Load => "%",
        SensorType.Control => "%",
        SensorType.Level => "%",
        SensorType.Humidity => "%",
        SensorType.Clock => "MHz",
        SensorType.Frequency => "Hz",
        SensorType.Power => "W",
        SensorType.Fan => "RPM",
        SensorType.Flow => "L/h",
        SensorType.Voltage => "V",
        SensorType.Factor => "1",
        SensorType.Data => "GB",
        SensorType.SmallData => "MB",
        SensorType.Throughput => "B/s",
        SensorType.TimeSpan => "s",
        SensorType.Timing => "ns",
        SensorType.Energy => "mWh",
        SensorType.Noise => "dBA",
        SensorType.Conductivity => "uS/cm",
        _ => null,
    };
}

public static class SensorValues
{
    public static double? Normalize(
        float? value,
        ICollection<string> errors,
        string identifier)
    {
        if (value is null)
        {
            return null;
        }

        var result = (double)value.Value;
        if (double.IsFinite(result))
        {
            return result;
        }

        errors.Add($"Sensor {identifier} returned a non-finite value.");
        return null;
    }
}
